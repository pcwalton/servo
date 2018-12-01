/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The layout thread. Performs layout on the DOM, builds display lists and sends them to be
//! painted.

#[macro_use]
extern crate crossbeam_channel;
#[macro_use]
extern crate html5ever;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

mod dom_wrapper;

use crossbeam_channel::Receiver;
use embedder_traits::resources::{self, Resource};
use euclid::TypedScale;
use ipc_channel::ipc::{self, IpcSender};
use ipc_channel::router::ROUTER;
use layout_traits::{LayoutGlobalInfo, LayoutThreadFactory};
use metrics::{PaintTimeMetrics, ProfilerMetadataFactory};
use msg::constellation_msg::TopLevelBrowsingContextId;
use profile_traits::time::{TimerMetadataFrameType, TimerMetadataReflowType, TimerMetadata};
use script_layout_interface::message::Msg;
use script_traits::{FrameType, LayoutControlMsg, LayoutPerThreadInfo};
use servo_arc::Arc as ServoArc;
use servo_config::opts;
use servo_url::ServoUrl;
use std::process;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use style::context::QuirksMode;
use style::error_reporting::RustLogReporter;
use style::media_queries::{Device, MediaList, MediaType};
use style::shared_lock::{SharedRwLock, SharedRwLockReadGuard};
use style::stylesheets::{DocumentStyleSheet, Origin, Stylesheet};
use style::stylesheets::{StylesheetInDocument, UserAgentStylesheets};
use style::stylist::Stylist;
use style::thread_state::{self, ThreadState};

/// The layout thread.
pub struct LayoutThread {
    /// General information shared among all layout threads.
    global_info: LayoutGlobalInfo,

    /// Specific information shared among all layout threads.
    thread_info: LayoutPerThreadInfo<Msg, PaintTimeMetrics>,

    /// Receivers that allow other threads to wake us up.
    messages: LayoutThreadMessages,

    /// Performs CSS selector matching and style resolution.
    stylist: Stylist,

    /// The number of Web fonts that have been requested but not yet loaded.
    outstanding_web_fonts: Arc<AtomicUsize>,
}

impl LayoutThreadFactory for LayoutThread {
    type Message = Msg;

    /// Spawns a new layout thread.
    fn create(thread_info: LayoutPerThreadInfo<Msg, PaintTimeMetrics>,
              global_info: LayoutGlobalInfo) {
        thread::Builder::new()
            .name(format!("LayoutThread {:?}", thread_info.local_pipeline_id))
            .spawn(move || {
                thread_state::initialize(ThreadState::LAYOUT);

                // In order to get accurate crash reports, we install the top-level bc id.
                TopLevelBrowsingContextId::install(global_info.top_level_context_id);

                let content_process_shutdown_sender =
                    thread_info.content_process_shutdown_sender.clone();

                {
                    let script_to_layout_sender = thread_info.script_to_layout_sender.clone();

                    // Ensures layout thread is destroyed before we send shutdown message
                    let reporter_name = format!("layout-reporter-{}",
                                                thread_info.local_pipeline_id);
                    let layout = LayoutThread::new(thread_info, global_info.clone());

                    global_info.mem_profiler_sender
                               .run_with_memory_reporting(|| layout.start(),
                                                          reporter_name,
                                                          script_to_layout_sender,
                                                          Msg::CollectReports);
                }

                if let Some(content_process_shutdown_sender) = content_process_shutdown_sender {
                    let _ = content_process_shutdown_sender.send(());
                }
            })
            .expect("Thread spawning failed");
    }
}

impl LayoutThread {
    /// Creates a new `LayoutThread` structure.
    fn new(thread_info: LayoutPerThreadInfo<Msg, PaintTimeMetrics>, global_info: LayoutGlobalInfo)
           -> LayoutThread {
        // The device pixel ratio is incorrect (it does not have the hidpi value),
        // but it will be set correctly when the initial reflow takes place.
        let device = Device::new(
            MediaType::screen(),
            opts::get().initial_window_size.to_f32() * TypedScale::new(1.0),
            TypedScale::new(opts::get().device_pixels_per_px.unwrap_or(1.0)),
        );

        // Create the channel on which new animations can be sent.
        // Proxy IPC messages from the pipeline to the layout thread.
        /*let pipeline_receiver =
            ROUTER.route_ipc_receiver_to_new_crossbeam_receiver(thread_info.pipeline_receiver
                                                                           .clone());*/

        // Ask the router to proxy IPC messages from the font cache thread to the layout thread.
        let (ipc_font_cache_sender, ipc_font_cache_receiver) = ipc::channel().unwrap();
        let font_cache_receiver =
            ROUTER.route_ipc_receiver_to_new_crossbeam_receiver(ipc_font_cache_receiver);

        LayoutThread {
            messages: LayoutThreadMessages {
                font_cache_receiver,
                font_cache_sender: ipc_font_cache_sender,
            },
            stylist: Stylist::new(device, QuirksMode::NoQuirks),
            outstanding_web_fonts: Arc::new(AtomicUsize::new(0)),
            global_info,
            thread_info,
        }
    }

    /// Starts listening on the port.
    fn start(mut self) {
        while self.handle_request() {
            // Loop indefinitely.
        }
    }

    /// Receives and dispatches messages from the script and constellation threads
    fn handle_request(&mut self) -> bool {
        enum Request {
            FromPipeline(LayoutControlMsg),
            FromScript(Msg),
            FromFontCache,
        }

        let request = select! {
            recv(self.thread_info.pipeline_receiver) -> msg => Request::FromPipeline(msg.unwrap()),
            recv(self.thread_info.script_to_layout_receiver) -> msg => {
                Request::FromScript(msg.unwrap())
            }
            recv(self.messages.font_cache_receiver) -> msg => {
                msg.unwrap();
                Request::FromFontCache
            }
        };

        match request {
            Request::FromPipeline(LayoutControlMsg::SetScrollStates(new_scroll_states)) => {
                self.handle_request_helper(Msg::SetScrollStates(new_scroll_states))
            }
            Request::FromPipeline(LayoutControlMsg::TickAnimations) => {
                self.handle_request_helper(Msg::TickAnimations)
            },
            Request::FromPipeline(LayoutControlMsg::GetCurrentEpoch(sender)) => {
                self.handle_request_helper(Msg::GetCurrentEpoch(sender))
            },
            Request::FromPipeline(LayoutControlMsg::GetWebFontLoadState(sender)) => self
                .handle_request_helper(Msg::GetWebFontLoadState(sender)),
            Request::FromPipeline(LayoutControlMsg::ExitNow) => {
                self.handle_request_helper(Msg::ExitNow)
            },
            Request::FromPipeline(LayoutControlMsg::PaintMetric(epoch, paint_time)) => {
                self.thread_info.paint_time_metrics.maybe_set_metric(epoch, paint_time);
                true
            },
            Request::FromScript(msg) => self.handle_request_helper(msg),
            Request::FromFontCache => {
                // TODO(pcwalton)
                true
            },
        }
    }

    /// Receives and dispatches messages from other threads.
    fn handle_request_helper(&mut self, request: Msg) -> bool {
        match request {
            Msg::AddStylesheet(stylesheet, before_stylesheet) => {
                let guard = stylesheet.shared_lock.read();
                self.handle_add_stylesheet(&stylesheet, &guard);

                match before_stylesheet {
                    Some(insertion_point) => self.stylist.insert_stylesheet_before(
                        DocumentStyleSheet(stylesheet.clone()),
                        DocumentStyleSheet(insertion_point),
                        &guard,
                    ),
                    None => {
                        self.stylist.append_stylesheet(DocumentStyleSheet(stylesheet.clone()),
                                                       &guard)
                    }
                }
            },
            Msg::RemoveStylesheet(stylesheet) => {
                let guard = stylesheet.shared_lock.read();
                self.stylist
                    .remove_stylesheet(DocumentStyleSheet(stylesheet.clone()), &guard);
            },
            Msg::SetQuirksMode(_mode) => {
                // TODO(pcwalton)
            }
            Msg::GetRPC(_response_sender) => {
                // TODO(pcwalton)
            },
            Msg::Reflow(_data) => {
                // TODO(pcwalton)
            },
            Msg::TickAnimations => {
                // TODO(pcwalton)
            }
            Msg::SetScrollStates(_new_scroll_states) => {
                // TODO(pcwalton)
            },
            Msg::UpdateScrollStateFromScript(_state) => {
                // TODO(pcwalton)
            },
            Msg::ReapStyleAndLayoutData(_dead_data) => {
                // TODO(pcwalton)
            }
            Msg::CollectReports(_reports_sender) => {
                // TODO(pcwalton)
            },
            Msg::GetCurrentEpoch(_sender) => {
                // TODO(pcwalton)
            }
            Msg::AdvanceClockMs(_how_many, _do_tick) => {
                // TODO(pcwalton)
            },
            Msg::GetWebFontLoadState(sender) => {
                let outstanding_web_fonts = self.outstanding_web_fonts.load(Ordering::SeqCst);
                sender.send(outstanding_web_fonts != 0).unwrap();
            },
            Msg::CreateLayoutThread(info) => self.create_layout_thread(info),
            Msg::SetFinalUrl(final_url) => self.thread_info.current_url = final_url,
            Msg::RegisterPaint(_name, _properties, _painter) => {
                // TODO(pcwalton)
            },
            Msg::PrepareToExit(_response_chan) => {
                // TODO(pcwalton)
                return false;
            },
            Msg::ExitNow => {
                // TODO(pcwalton)
                debug!("layout: ExitNow received");
                return false;
            }
            Msg::SetNavigationStart(_time) => {
                // TODO(pcwalton)
            }
        }

        true
    }

    fn create_layout_thread(&self, thread_info: LayoutPerThreadInfo<Msg, PaintTimeMetrics>) {
        LayoutThread::create(thread_info, self.global_info.clone())
    }

    fn handle_add_stylesheet(&self, stylesheet: &Stylesheet, guard: &SharedRwLockReadGuard) {
        // Find all font-face rules and notify the font cache of them.
        // GWTODO: Need to handle unloading web fonts.
        if stylesheet.is_effective_for_device(self.stylist.device(), &guard) {
            self.add_font_face_rules(&*stylesheet, &guard);
        }
    }

    fn add_font_face_rules(&self, stylesheet: &Stylesheet, guard: &SharedRwLockReadGuard) {
        if opts::get().load_webfonts_synchronously {
            let (sender, receiver) = ipc::channel().unwrap();
            stylesheet.effective_font_face_rules(&self.stylist.device(), guard, |rule| {
                if let Some(font_face) = rule.font_face() {
                    let effective_sources = font_face.effective_sources();
                    self.global_info.font_cache_thread.add_web_font(
                        font_face.family().clone(),
                        effective_sources,
                        sender.clone(),
                    );
                    receiver.recv().unwrap();
                }
            })
        } else {
            stylesheet.effective_font_face_rules(&self.stylist.device(), guard, |rule| {
                if let Some(font_face) = rule.font_face() {
                    let effective_sources = font_face.effective_sources();
                    self.outstanding_web_fonts.fetch_add(1, Ordering::SeqCst);
                    self.global_info.font_cache_thread.add_web_font(
                        font_face.family().clone(),
                        effective_sources,
                        self.messages.font_cache_sender.clone(),
                    );
                }
            })
        }
    }

    /// Returns profiling information which is passed to the time profiler.
    fn profiler_metadata(&self) -> Option<TimerMetadata> {
        Some(TimerMetadata {
            url: self.thread_info.current_url.to_string(),
            iframe: match self.thread_info.frame_type {
                FrameType::IFrame => TimerMetadataFrameType::IFrame,
                FrameType::RootWindow => TimerMetadataFrameType::RootWindow,
            },
            incremental: TimerMetadataReflowType::FirstReflow,
        })
    }
}

impl ProfilerMetadataFactory for LayoutThread {
    fn new_metadata(&self) -> Option<TimerMetadata> {
        self.profiler_metadata()
    }
}

fn get_ua_stylesheets() -> Result<UserAgentStylesheets, &'static str> {
    fn parse_ua_stylesheet(
        shared_lock: &SharedRwLock,
        filename: &str,
        content: &[u8],
    ) -> Result<DocumentStyleSheet, &'static str> {
        Ok(DocumentStyleSheet(ServoArc::new(Stylesheet::from_bytes(
            content,
            ServoUrl::parse(&format!("chrome://resources/{:?}", filename)).unwrap(),
            None,
            None,
            Origin::UserAgent,
            MediaList::empty(),
            shared_lock.clone(),
            None,
            None,
            QuirksMode::NoQuirks,
        ))))
    }

    let shared_lock = SharedRwLock::new();
    // FIXME: presentational-hints.css should be at author origin with zero specificity.
    //        (Does it make a difference?)
    let mut user_or_user_agent_stylesheets = vec![
        parse_ua_stylesheet(
            &shared_lock,
            "user-agent.css",
            &resources::read_bytes(Resource::UserAgentCSS),
        )?,
        parse_ua_stylesheet(
            &shared_lock,
            "servo.css",
            &resources::read_bytes(Resource::ServoCSS),
        )?,
        parse_ua_stylesheet(
            &shared_lock,
            "presentational-hints.css",
            &resources::read_bytes(Resource::PresentationalHintsCSS),
        )?,
    ];

    for &(ref contents, ref url) in &opts::get().user_stylesheets {
        user_or_user_agent_stylesheets.push(DocumentStyleSheet(ServoArc::new(
            Stylesheet::from_bytes(
                &contents,
                url.clone(),
                None,
                None,
                Origin::User,
                MediaList::empty(),
                shared_lock.clone(),
                None,
                Some(&RustLogReporter),
                QuirksMode::NoQuirks,
            ),
        )));
    }

    let quirks_mode_stylesheet = parse_ua_stylesheet(
        &shared_lock,
        "quirks-mode.css",
        &resources::read_bytes(Resource::QuirksModeCSS),
    )?;

    Ok(UserAgentStylesheets {
        shared_lock: shared_lock,
        user_or_user_agent_stylesheets: user_or_user_agent_stylesheets,
        quirks_mode_stylesheet: quirks_mode_stylesheet,
    })
}

lazy_static! {
    static ref UA_STYLESHEETS: UserAgentStylesheets = {
        match get_ua_stylesheets() {
            Ok(stylesheets) => stylesheets,
            Err(filename) => {
                error!("Failed to load UA stylesheet {}!", filename);
                process::exit(1);
            },
        }
    };
}

struct LayoutThreadMessages {
    /// A receiver that lets the font cache thread wake us up.
    font_cache_receiver: Receiver<()>,

    /// A sender to the previous channel.
    font_cache_sender: IpcSender<()>,
}
