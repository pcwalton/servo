/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! CSS transitions and animations.

use flow::{mod, Flow};
use fragment::{Fragment, FragmentBoundsIterator};
use incremental::{mod, RestyleDamage};

use geom::rect::Rect;
use gfx::display_list::OpaqueNode;
use script::layout_interface::{Animation, LayoutChan, UpdateAnimationMsg};
use servo_util::geometry::Au;
use std::comm::{mod, Receiver, Sender};
use std::io::timer;
use std::mem;
use std::task;
use std::time::Duration;
use style::ComputedValues;
use style::animation::PropertyAnimation;
use time;

/// Data stored by the animation thread. There is at most one of these per layout thread.
pub struct AnimationThread {
    /// A list of currently-playing animations.
    animations: Vec<Animation>,
    /// Receives messages inbound to the animation thread.
    receiver: Receiver<Msg>,
    /// A channel to the layout task for us to send updates to.
    layout_proxy: LayoutChan,
}

impl AnimationThread {
    /// Creates and starts a new animation thread.
    pub fn start(layout_proxy: LayoutChan) -> Sender<Msg> {
        let (sender, receiver) = comm::channel();
        task::spawn(proc() {
            let mut thread = AnimationThread {
                animations: Vec::new(),
                receiver: receiver,
                layout_proxy: layout_proxy,
            };
            while thread.tick() {}
        });
        sender
    }

    /// Handles and processes inbound messages. Returns true if we are to continue or false if we
    /// are to stop.
    ///
    /// FIXME(pcwalton): This is a VERY bad refresh driver. Do better. Really we should sync to
    /// buffer swaps in the compositor.
    /// FIXME(pcwalton): Sleep if no animations. Don't poll.
    pub fn tick(&mut self) -> bool {
        // Process messages.
        loop {
            match self.receiver.try_recv() {
                Ok(ScheduleMsg(animation)) => self.animations.push(animation),
                Ok(ExitMsg) => return false,
                Err(_) => break,
            }
        }

        // Handle animation updates.
        let animations = mem::replace(&mut self.animations, Vec::new());
        let now = time::precise_time_s();
        for animation in animations.into_iter() {
            let LayoutChan(ref mut layout_proxy) = self.layout_proxy;
            layout_proxy.send(UpdateAnimationMsg(box animation));
            if now < animation.end_time {
                // Keep running the animation if it hasn't expired.
                self.animations.push(animation)
            }
        }

        // Sleep.
        //
        // FIXME(pcwalton): Don't.
        timer::sleep(Duration::seconds(1) / 60);
        true
    }
}

/// Messages that can be sent to the animation thread.
pub enum Msg {
    /// Schedules the given animation to play.
    ///
    /// TODO(pcwalton): Add a delay. Right now we just start playing animations right away.
    ScheduleMsg(Animation),
    /// Tells the animation thread to shut down.
    ExitMsg,
}

/// Kicks off transitions for the given style difference. This is called from the layout worker
/// threads.
pub fn start_transitions_if_applicable(animation_thread_proxy: &Sender<Msg>,
                                       node: OpaqueNode,
                                       old_style: &ComputedValues,
                                       new_style: &mut ComputedValues) {
    // Create the property animation, if applicable. If the property in question was not changed,
    // this will return out.
    let property_animation =
        match PropertyAnimation::new(new_style.get_animation().transition_property,
                                     old_style,
                                     new_style) {
            None => return,
            Some(property_animation) => property_animation,
        };

    // Kick off the animation.
    let now = time::precise_time_s();
    animation_thread_proxy.send(ScheduleMsg(Animation {
        node: node.id(),
        property_animation: property_animation,
        start_time: now,
        end_time: now + new_style.get_animation().transition_duration.seconds(),
    }))
}

/// Recalculates style for an animation. This does *not* run with the DOM lock held.
pub fn recalc_style_for_animation(flow: &mut Flow, animation: &Animation) {
    let mut damage = RestyleDamage::empty();
    flow.iterate_through_fragment_bounds(&mut AnimationFragmentIterator {
        damage: &mut damage,
        animation: animation,
    });

    let base = flow::mut_base(flow);
    base.restyle_damage.insert(damage);
    for kid in base.children.iter_mut() {
        recalc_style_for_animation(kid, animation)
    }
}

pub struct AnimationFragmentIterator<'a> {
    damage: &'a mut RestyleDamage,
    animation: &'a Animation,
}

impl<'a> FragmentBoundsIterator for AnimationFragmentIterator<'a> {
    fn process(&mut self, fragment: &mut Fragment, _: Rect<Au>) {
        if fragment.node.id() != self.animation.node {
            return
        }
        let now = time::precise_time_s();
        let mut progress = (now - self.animation.start_time) / self.animation.duration();
        if progress > 1.0 {
            progress = 1.0
        }

        let new_style = self.animation
                            .property_animation
                            .update((*fragment.style.clone()).clone(), progress);
        self.damage.insert(incremental::compute_damage(&Some(fragment.style.clone()),
                                                       &new_style));
        *fragment.style.make_unique() = new_style
    }

    fn should_process(&mut self, _: &Fragment) -> bool {
        true
    }
}

