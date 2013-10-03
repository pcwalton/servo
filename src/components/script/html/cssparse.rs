/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// Some little helpers for hooking up the HTML parser with the CSS parser.

use std::cell::Cell;
use std::comm;
use std::comm::Port;
use std::task;
use newcss::stylesheet::Stylesheet;
use newcss::util::DataStream;
use servo_net::resource_task::{ResourceTask, ProgressMsg, Load, Payload, Done, UrlChange};
use extra::url::Url;

/// Where a style sheet comes from.
pub enum StylesheetProvenance {
    UrlProvenance(Url),
    InlineProvenance(Url, ~str),
}

pub fn spawn_css_parser(provenance: StylesheetProvenance,
                        resource_task: ResourceTask)
                     -> Port<Stylesheet> {
    let (result_port, result_chan) = comm::stream();

    let provenance_cell = Cell::new(provenance);
    do task::spawn {
        let url = do provenance_cell.with_ref |p| {
            match *p {
                UrlProvenance(ref the_url) => (*the_url).clone(),
                InlineProvenance(ref the_url, _) => (*the_url).clone()
            }
        };

        let sheet = Stylesheet::new(url, data_stream(provenance_cell.take(),
                                                     resource_task.clone()));
        result_chan.send(sheet);
    }

    return result_port;
}

fn data_stream(provenance: StylesheetProvenance, resource_task: ResourceTask) -> @mut DataStream {
    match provenance {
        UrlProvenance(url) => {
            debug!("cssparse: loading style sheet at %s", url.to_str());
            let (input_port, input_chan) = comm::stream();
            resource_task.send(Load(url, input_chan));
            resource_port_to_data_stream(input_port)
        }
        InlineProvenance(_, data) => {
            data_to_data_stream(data)
        }
    }
}

struct ResourcePort {
    input_port: Port<ProgressMsg>,
}

impl DataStream for ResourcePort {
    fn read(&mut self) -> Option<~[u8]> {
        loop {
            match self.input_port.recv() {
                UrlChange(*) => (),  // don't care that URL changed
                Payload(data) => return Some(data),
                Done(*) => break
            }
        }
        None
    }
}

fn resource_port_to_data_stream(input_port: Port<ProgressMsg>) -> @mut DataStream {
    @mut ResourcePort {
        input_port: input_port,
    } as @mut DataStream
}

struct Data {
    data: Option<~str>,
}

impl DataStream for Data {
    fn read(&mut self) -> Option<~[u8]> {
        if self.data.is_none() {
            None
        } else {
            let data = self.data.take_unwrap();
            Some(data.as_bytes().to_owned())
        }
    }
}

fn data_to_data_stream(data: ~str) -> @mut DataStream {
    @mut Data {
        data: Some(data),
    } as @mut DataStream
}

