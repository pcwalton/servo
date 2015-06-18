/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use CompositorProxy;
use content_process::{mod, AuxiliaryContentProcessData, ContentProcess, ContentProcessIpc, Zone};
use layout_traits::{LayoutControlMsg, LayoutTaskFactory, LayoutControlChan};
use script_traits::{ScriptControlChan, ScriptTaskFactory};
use script_traits::{ConstellationControlMsg};

use devtools_traits::DevtoolsControlChan;
use geom::rect::{TypedRect};
use geom::scale_factor::ScaleFactor;
use gfx::paint_task::Msg as PaintMsg;
use gfx::paint_task::{PaintChan, PaintTask};
use gfx::font_cache_task::FontCacheTask;
use layers::geometry::DevicePixel;
use msg::constellation_msg::{ConstellationChan, Failure, FrameId, PipelineId, SubpageId};
use msg::constellation_msg::{LoadData, WindowSizeData, PipelineExitType, MozBrowserEvent};
use profile_traits::mem;
use profile_traits::time;
use net_traits::ResourceTask;
use net_traits::image_cache_task::ImageCacheTask;
use net_traits::storage_task::StorageTask;
use std::sync::mpsc::{Receiver, channel};
use url::Url;
use util::geometry::{PagePx, ViewportPx};
use util::ipc;
use util::opts;

/// A uniquely-identifiable pipeline of script task, layout task, and paint task.
pub struct Pipeline {
    pub id: PipelineId,
    pub parent_info: Option<(PipelineId, SubpageId)>,
    pub script_chan: ScriptControlChan,
    /// A channel to layout, for performing reflows and shutdown.
    pub layout_chan: LayoutControlChan,
    pub paint_chan: PaintChan,
    pub layout_shutdown_port: Receiver<()>,
    pub paint_shutdown_port: Receiver<()>,
    /// URL corresponding to the most recently-loaded page.
    pub url: Url,
    /// Load data corresponding to the most recently-loaded page.
    pub load_data: LoadData,
    /// The title of the most recently-loaded page.
    pub title: Option<String>,
    pub rect: Option<TypedRect<PagePx, f32>>,
    /// Whether this pipeline is currently running animations. Pipelines that are running
    /// animations cause composites to be continually scheduled.
    pub running_animations: bool,
    pub children: Vec<FrameId>,
}

/// The subset of the pipeline that is needed for layer composition.
#[derive(Clone)]
pub struct CompositionPipeline {
    pub id: PipelineId,
    pub script_chan: ScriptControlChan,
    pub layout_chan: LayoutControlChan,
    pub paint_chan: PaintChan,
}

impl Pipeline {
    /// Starts a paint task, layout task, and possibly a script task.
    /// Returns the channels wrapped in a struct.
    /// If script_pipeline is not None, then subpage_id must also be not None.
    pub fn create<LTF,STF>(id: PipelineId,
                           parent_info: Option<(PipelineId, SubpageId)>,
                           constellation_chan: ConstellationChan,
                           compositor_proxy: Box<CompositorProxy+'static+Send>,
                           script_to_compositor_client: SharedServerProxy<ScriptToCompositorMsg,
                                                                          ()>,
                           _: Option<DevtoolsControlChan>,
                           image_cache_task: ImageCacheTask,
                           font_cache_task: FontCacheTask,
                           resource_task: ResourceTask,
                           storage_task: StorageTask,
                           time_profiler_chan: time::ProfilerChan,
                           mem_profiler_chan: mem::ProfilerChan,
                           window_rect: Option<TypedRect<PagePx, f32>>,
                           script_chan: Option<ScriptControlChan>,
                           load_data: LoadData,
                           device_pixel_ratio: ScaleFactor<ViewportPx, DevicePixel, f32>)
                           -> Pipeline
                           where LTF: LayoutTaskFactory, STF:ScriptTaskFactory {
        let (paint_port, paint_chan) = PaintChan::new();
        let (_, layout_shutdown_port) = channel();
        let (pipeline_port, pipeline_chan) = ipc::channel();

        let failure = Failure {
            pipeline_id: id,
            parent_info: parent_info,
        };

        let (script_port, script_chan) = ipc::channel();
        let content_process_ipc = ContentProcessIpc {
            script_to_compositor_client: script_to_compositor_client,
            script_port: script_port,
            constellation_chan: constellation_chan.clone(),
            storage_task: storage_task,
            pipeline_to_layout_port: pipeline_port,
            layout_to_paint_chan: paint_chan.create_layout_channel(),
            font_cache_task: font_cache_task.clone(),
        };

        match script_pipeline {
            None => {
                let data = AuxiliaryContentProcessData {
                    pipeline_id: id,
                    failure: failure,
                    window_size: window_size,
                    zone: Zone::from_load_data(&load_data),
                };

                if !opts::get().multiprocess {
                    let content_process = ContentProcess {
                        ipc: content_process_ipc,
                        resource_task: resource_task.clone(),
                        image_cache_task: image_cache_task.clone(),
                        time_profiler_chan: time_profiler_chan.clone(),
                    };
                    content_process.create_script_and_layout_threads(data)
                } else {
                    content_process::spawn(content_process_ipc, data)
                }
            }
            Some(_spipe) => {
                panic!("layout connection to existing script thread not yet ported to e10s")
            }
        }

        PaintTask::create(id,
                          load_data.url.clone(),
                          paint_chan.clone(),
                          paint_port,
                          compositor_proxy,
                          constellation_chan.clone(),
                          font_cache_task,
                          failure.clone(),
                          time_profiler_chan.clone());

        Pipeline::new(id,
                      subpage_id,
                      ScriptControlChan(script_chan),
                      LayoutControlChan(pipeline_chan),
                      paint_chan,
                      layout_shutdown_port,
                      load_data)
    }

    pub fn new(id: PipelineId,
               parent_info: Option<(PipelineId, SubpageId)>,
               script_chan: ScriptControlChan,
               layout_chan: LayoutControlChan,
               paint_chan: PaintChan,
               layout_shutdown_port: Receiver<()>,
               load_data: LoadData)
               -> Pipeline {
        Pipeline {
            id: id,
            parent_info: parent_info,
            script_chan: script_chan,
            layout_chan: layout_chan,
            paint_chan: paint_chan,
            layout_shutdown_port: layout_shutdown_port,
            load_data: load_data,
            title: None,
            children: vec!(),
            rect: rect,
            running_animations: false,
        }
    }

    pub fn grant_paint_permission(&self) {
        let _ = self.paint_chan.send(PaintMsg::PaintPermissionGranted);
    }

    pub fn revoke_paint_permission(&self) {
        debug!("pipeline revoking paint channel paint permission");
        let _ = self.paint_chan.send(PaintMsg::PaintPermissionRevoked);
    }

    pub fn exit(&self, exit_type: PipelineExitType) {
        debug!("pipeline {:?} exiting", self.id);

        // Script task handles shutting down layout, and layout handles shutting down the painter.
        // For now, if the script task has failed, we give up on clean shutdown.
        let ScriptControlChan(ref chan) = self.script_chan;
        if chan.send(ConstellationControlMsg::ExitPipeline(self.id, exit_type)).is_ok() {
            // Wait until all slave tasks have terminated and run destructors
            // NOTE: We don't wait for script task as we don't always own it
            let _ = self.layout_shutdown_port.recv_opt();
        }

    }

    pub fn freeze(&self) {
        let ScriptControlChan(ref script_channel) = self.script_chan;
        let _ = script_channel.send(ConstellationControlMsg::Freeze(self.id)).unwrap();
    }

    pub fn thaw(&self) {
        let ScriptControlChan(ref script_channel) = self.script_chan;
        let _ = script_channel.send(ConstellationControlMsg::Thaw(self.id)).unwrap();
    }

    pub fn force_exit(&self) {
        let ScriptControlChan(ref script_channel) = self.script_chan;
        let _ = script_channel.send(
            ConstellationControlMsg::ExitPipeline(self.id,
                                                  PipelineExitType::PipelineOnly));
        let _ = self.paint_chan.send_opt(PaintMsg::Exit(PipelineExitType::PipelineOnly));
        let LayoutControlChan(ref layout_channel) = self.layout_chan;
        let _ = layout_channel.send(
            LayoutControlMsg::ExitNow(PipelineExitType::PipelineOnly)).unwrap();
    }

    pub fn to_sendable(&self) -> CompositionPipeline {
        CompositionPipeline {
            id: self.id.clone(),
            script_chan: self.script_chan.clone(),
            layout_chan: self.layout_chan.clone(),
            paint_chan: self.paint_chan.clone(),
        }
    }

    pub fn add_child(&mut self, frame_id: FrameId) {
        self.children.push(frame_id);
    }

    pub fn remove_child(&mut self, frame_id: FrameId) {
        let index = self.children.iter().position(|id| *id == frame_id).unwrap();
        self.children.remove(index);
    }

    pub fn trigger_mozbrowser_event(&self,
                                     subpage_id: SubpageId,
                                     event: MozBrowserEvent) {
        assert!(opts::experimental_enabled());

        let ScriptControlChan(ref script_channel) = self.script_chan;
        let event = ConstellationControlMsg::MozBrowserEvent(self.id,
                                                             subpage_id,
                                                             event);
        script_channel.send(event).unwrap();
    }
}
