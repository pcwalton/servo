/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Implements parallel traversals over the DOM and flow trees.

use crate::block::BlockFlow;
use crate::context::LayoutContext;
use crate::flow::{Flow, GetBaseFlow};
use crate::flow_ref::FlowRef;
use crate::traversal::{AssignBSizes, AssignISizes, BubbleISizes};
use crate::traversal::{PostorderFlowTraversal, PreorderFlowTraversal};
use profile_traits::time::{self, profile, TimerMetadata};
use servo_config::opts;
use smallvec::SmallVec;
use std::mem;
use std::ptr;
use std::sync::atomic::{AtomicIsize, Ordering};

/// Traversal chunk size.
const CHUNK_SIZE: usize = 16;

pub type FlowList = SmallVec<[FlowRef; CHUNK_SIZE]>;

/// Information that we need stored in each flow.
pub struct FlowParallelInfo {
    /// The number of children that still need work done.
    pub children_count: AtomicIsize,
    /// The address of the parent flow.
    pub parent: Option<FlowRef>,
}

impl FlowParallelInfo {
    pub fn new() -> FlowParallelInfo {
        FlowParallelInfo {
            children_count: AtomicIsize::new(0),
            parent: None,
        }
    }
}

/// Process current flow and potentially traverse its ancestors.
///
/// If we are the last child that finished processing, recursively process
/// our parent. Else, stop. Also, stop at the root.
///
/// Thus, if we start with all the leaves of a tree, we end up traversing
/// the whole tree bottom-up because each parent will be processed exactly
/// once (by the last child that finishes processing).
///
/// The only communication between siblings is that they both
/// fetch-and-subtract the parent's children count.
fn bottom_up_flow(mut flow_ref: FlowRef, assign_bsize_traversal: &AssignBSizes) {
    loop {
        let parent;
        {
            // Get a real flow.
            let mut flow = flow_ref.write();

            if assign_bsize_traversal.should_process(&mut *flow) {
                assign_bsize_traversal.process(&mut *flow);
            }

            let base = flow.mut_base();

            // Reset the count of children for the next layout traversal.
            base.parallel
                .children_count
                .store(base.children.len() as isize, Ordering::Relaxed);

            // Possibly enqueue the parent.
            parent = match base.parallel.parent {
                None => {
                    // We're done!
                    break;
                }
                Some(ref parent) => (*parent).clone(),
            };
        }

        // No, we're not at the root yet. Then are we the last child
        // of our parent to finish processing? If so, we can continue
        // on with our parent; otherwise, we've gotta wait.
        if parent.write().mut_base().parallel.children_count.fetch_sub(1, Ordering::Relaxed) == 1 {
            // We were the last child of our parent. Reflow our parent.
            flow_ref = parent
        } else {
            // Stop.
            break;
        }
    }
}

fn top_down_flow<'scope>(
    flow_refs: &[FlowRef],
    pool: &'scope rayon::ThreadPool,
    scope: &rayon::Scope<'scope>,
    assign_isize_traversal: &'scope AssignISizes,
    assign_bsize_traversal: &'scope AssignBSizes,
) {
    let mut discovered_child_flows = FlowList::new();

    for flow_ref in flow_refs {
        let mut had_children = false;

        // Get a real flow.
        let mut flow = flow_ref.write();

        if assign_isize_traversal.should_process(&mut *flow) {
            // Perform the appropriate traversal.
            assign_isize_traversal.process(&mut *flow);
        }

        // Possibly enqueue the children.
        for kid in flow.mut_base().child_iter() {
            had_children = true;
            discovered_child_flows.push(kid.clone());
        }

        // If there were no more children, start assigning block-sizes.
        if !had_children {
            bottom_up_flow((*flow_ref).clone(), &assign_bsize_traversal)
        }
    }

    if discovered_child_flows.is_empty() {
        return;
    }

    if discovered_child_flows.len() <= CHUNK_SIZE {
        // We can handle all the children in this work unit.
        top_down_flow(
            &discovered_child_flows,
            pool,
            scope,
            &assign_isize_traversal,
            &assign_bsize_traversal,
        );
    } else {
        // Spawn a new work unit for each chunk after the first.
        let mut chunks = discovered_child_flows.chunks(CHUNK_SIZE);
        let first_chunk = chunks.next();
        for chunk in chunks {
            let nodes = chunk.iter().cloned().collect::<FlowList>();
            scope.spawn(move |scope| {
                top_down_flow(
                    &nodes,
                    pool,
                    scope,
                    &assign_isize_traversal,
                    &assign_bsize_traversal,
                );
            });
        }
        if let Some(chunk) = first_chunk {
            top_down_flow(
                chunk,
                pool,
                scope,
                &assign_isize_traversal,
                &assign_bsize_traversal,
            );
        }
    }
}

/// Run the main layout passes in parallel.
pub fn reflow(
    root: FlowRef,
    profiler_metadata: Option<TimerMetadata>,
    time_profiler_chan: time::ProfilerChan,
    context: &LayoutContext,
    queue: &rayon::ThreadPool,
) {
    if opts::get().bubble_inline_sizes_separately {
        let bubble_inline_sizes = BubbleISizes {
            layout_context: &context,
        };
        bubble_inline_sizes.traverse(&mut *root.write());
    }

    let assign_isize_traversal = &AssignISizes {
        layout_context: &context,
    };
    let assign_bsize_traversal = &AssignBSizes {
        layout_context: &context,
    };
    let nodes = [root];

    queue.install(move || {
        rayon::scope(move |scope| {
            profile(
                time::ProfilerCategory::LayoutParallelWarmup,
                profiler_metadata,
                time_profiler_chan,
                move || {
                    top_down_flow(
                        &nodes,
                        queue,
                        scope,
                        assign_isize_traversal,
                        assign_bsize_traversal,
                    );
                },
            );
        });
    });
}
