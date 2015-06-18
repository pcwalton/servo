/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::borrow::ToOwned;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use url::Url;

use net_traits::storage_task::{StorageTask, StorageTaskMsg, StorageType};
use util::str::DOMString;
use util::task::spawn_named;

/// Handle to a storage task
#[deriving(Clone)]
pub struct StorageTask {
    pub client: SharedServerProxy<StorageTaskMsg,StorageTaskResponse>,
}

impl StorageTask {
    #[inline]
    pub fn from_client(client: SharedServerProxy<StorageTaskMsg,StorageTaskResponse>)
                       -> StorageTask {
        StorageTask {
            client: client,
        }
    }

    pub fn send(&self, msg: StorageTaskMsg) -> StorageTaskResponse {
        self.client.lock().send_sync(msg)
    }

    pub fn create_new_client(&self) -> StorageTask {
        StorageTask {
            client: Arc::new(Mutex::new(self.client.lock().create_new_client())),
        }
    }
}

pub trait StorageTaskFactory {
    fn new() -> Self;
}

impl StorageTaskFactory for StorageTask {
    /// Create a StorageTask
    fn new() -> StorageTask {
        let mut server = Server::new("StorageTask");
        let client = Arc::new(Mutex::new(server.create_new_client()));
        spawn_named("StorageManager".to_owned(), proc() {
            StorageManager::new(server).start();
        });
        StorageTask {
            client: client,
        }
    }
}

struct StorageManager {
    server: Server<StorageTaskMsg,StorageTaskResponse>,
    session_data: HashMap<String, BTreeMap<DOMString, DOMString>>,
    local_data: HashMap<String, BTreeMap<DOMString, DOMString>>,
}

impl StorageManager {
    fn new(server: Server<StorageTaskMsg,StorageTaskResponse>) -> StorageManager {
        StorageManager {
            server: server,
            session_data: HashMap::new(),
            local_data: HashMap::new(),
        }
    }
}

impl StorageManager {
    fn start(&mut self) {
        while let Some(msgs) = self.server.recv() {
            for (client_id, msg) in msgs.into_iter() {
                match msg {
                    StorageTaskMsg::Length(url) => self.length(client_id, url),
                    StorageTaskMsg::Key(url, index) => self.key(client_id, url, index),
                    StorageTaskMsg::SetItem(url, name, value) => {
                        self.set_item(client_id, url, name, value)
                    }
                    StorageTaskMsg::GetItem(url, name) => self.get_item(client_id, url, name),
                    StorageTaskMsg::RemoveItem(url, name) => {
                        self.remove_item(client_id, url, name)
                    }
                    StorageTaskMsg::Clear(url) => self.clear(client_id, url),
                }
            }
        }
    }

    fn select_data(& self, storage_type: StorageType) -> &HashMap<String, BTreeMap<DOMString, DOMString>> {
        match storage_type {
            StorageType::Session => &self.session_data,
            StorageType::Local => &self.local_data
        }
    }

    fn select_data_mut(&mut self, storage_type: StorageType) -> &mut HashMap<String, BTreeMap<DOMString, DOMString>> {
        match storage_type {
            StorageType::Session => &mut self.session_data,
            StorageType::Local => &mut self.local_data
        }
    }

    fn length(&self, sender: Sender<usize>, url: Url, storage_type: StorageType) {
        let origin = self.get_origin_as_string(url);
        let data = self.select_data(storage_type);
        sender.send(data.get(&origin).map_or(0, |entry| entry.len())).unwrap();
    }

    fn key(&self, sender: Sender<Option<DOMString>>, url: Url, storage_type: StorageType, index: u32) {
        let origin = self.get_origin_as_string(url);
        let data = self.select_data(storage_type);
        sender.send(data.get(&origin)
                    .and_then(|entry| entry.keys().nth(index as usize))
                    .map(|key| key.clone())).unwrap();
    }

    /// Sends Some(old_value) in case there was a previous value with the same key name but with different
    /// value name, otherwise sends None
    fn set_item(&mut self, sender: Sender<(bool, Option<DOMString>)>, url: Url, storage_type: StorageType,
                name: DOMString, value: DOMString) {
        let origin = self.get_origin_as_string(url);
        let data = self.select_data_mut(storage_type);
        if !data.contains_key(&origin) {
            data.insert(origin.clone(), BTreeMap::new());
        }

        let (changed, old_value) = data.get_mut(&origin).map(|entry| {
            entry.insert(name, value.clone()).map_or(
                (true, None),
                |old| if old == value {
                    (false, None)
                } else {
                    (true, Some(old))
                })
        }).unwrap();
        sender.send((changed, old_value)).unwrap();
    }

    fn get_item(&self, sender: Sender<Option<DOMString>>, url: Url, storage_type: StorageType, name: DOMString) {
        let origin = self.get_origin_as_string(url);
        let data = self.select_data(storage_type);
        sender.send(data.get(&origin)
                    .and_then(|entry| entry.get(&name))
                    .map(|value| value.to_string())).unwrap();
    }

    /// Sends Some(old_value) in case there was a previous value with the key name, otherwise sends None
    fn remove_item(&mut self, sender: Sender<Option<DOMString>>, url: Url, storage_type: StorageType,
                   name: DOMString) {
        let origin = self.get_origin_as_string(url);
        let data = self.select_data_mut(storage_type);
        let old_value = data.get_mut(&origin).and_then(|entry| {
            entry.remove(&name)
        });
        sender.send(old_value).unwrap();
    }

    fn clear(&mut self, sender: Sender<bool>, url: Url, storage_type: StorageType) {
        let origin = self.get_origin_as_string(url);
        let data = self.select_data_mut(storage_type);
        sender.send(data.get_mut(&origin)
                    .map_or(false, |entry| {
                        if !entry.is_empty() {
                            entry.clear();
                            true
                        } else {
                            false
                        }})).unwrap();
    }

    fn get_origin_as_string(&self, url: Url) -> String {
        let mut origin = "".to_string();
        origin.push_str(&url.scheme);
        origin.push_str("://");
        url.domain().map(|domain| origin.push_str(&domain));
        url.port().map(|port| {
            origin.push_str(":");
            origin.push_str(&port.to_string());
        });
        origin.push_str("/");
        origin
    }
}
