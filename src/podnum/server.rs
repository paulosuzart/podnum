use std::collections::HashMap;
use std::hash::Hash;
use std::time::Duration;
use commitlog::LogOptions;
use omnipaxos::{ClusterConfig, OmniPaxos, OmniPaxosConfig, ServerConfig};
use omnipaxos::storage::{Entry, Snapshot};
use omnipaxos_storage::persistent_storage::{PersistentStorage, PersistentStorageConfig};
use sled::{Config};
use serde::{Deserialize, Serialize};
use tokio::time;
use tracing::info;
use crate::podnum::network::Network;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssignEntry {
    host: String,
    num: u32,
    timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KVCommand {
    Assign(AssignEntry),
    UnAssign(u32),
}

impl Entry for KVCommand {
    type Snapshot = KVSnapshot;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KVSnapshot {
    snapshotted: HashMap<u32, AssignEntry>,
    deleted_keys: Vec<u32>,
}

impl Snapshot<KVCommand> for KVSnapshot {
    fn create(entries: &[KVCommand]) -> Self {
        let mut snapshotted = HashMap::new();
        let mut deleted_keys: Vec<u32> = Vec::new();
        for e in entries {
            match e {
                KVCommand::Assign(a) => {
                    snapshotted.insert(a.num.clone(), a.clone());
                }
                KVCommand::UnAssign(key) => {
                    if snapshotted.remove(key).is_none() {
                        // key was not in the snapshot
                        deleted_keys.push(key.clone());
                    }
                }
            }
        }
        // remove keys that were put back
        deleted_keys.retain(|k| !snapshotted.contains_key(k));
        Self {
            snapshotted,
            deleted_keys,
        }
    }

    fn merge(&mut self, delta: Self) {
        for (k, v) in delta.snapshotted {
            self.snapshotted.insert(k, v);
        }
        for k in delta.deleted_keys {
            self.snapshotted.remove(&k);
        }
        self.deleted_keys.clear();
    }

    fn use_snapshots() -> bool {
        true
    }
}

type OmniPaxosKV = OmniPaxos<KVCommand, PersistentStorage<KVCommand>>;

struct Database {
    data: HashMap<u32, AssignEntry>,
}

impl Database {
    pub fn new() -> Self {
        Self {
            data: HashMap::new()
        }
    }

    pub fn handle_command(&mut self, command: KVCommand) {
        match command {
            KVCommand::Assign(a) => {
                self.data.insert(a.num, a);
            }
            KVCommand::UnAssign(host) => {
                self.data.remove(&host);
            }
        }
    }

    pub fn get_podnum(self, host: String) -> Option<u32> {
        self.data.values()
            .find(|x| x.host == host)
            .map(|x| x.num)
    }
}

pub struct Server {
    omni_paxos: OmniPaxosKV,
    database: Database,
    last_decided_idx: u64,
    pid: u64,
}

impl Server {
    pub fn get_podnum(&self, host: &String) -> u64 {
        if self.is_leader() {
            1
        } else {
            2
        }
    }

    fn is_leader(&self) -> bool {
        let leader = self.omni_paxos.get_current_leader().unwrap();
        let pid = self.pid;
        pid == leader
    }

    pub(crate) async fn run(&mut self) {
        info!("Running server");
        let mut tick_interval = time::interval(Duration::from_millis(5000));
        loop {
            tokio::select! {
                biased;
                _ = tick_interval.tick() => {
                    self.omni_paxos.tick();
                }
            }
        }
    }
}

//     async fn process_incoming_msgs(&mut self) {
//         let messages = self.network.get_received().await;
//         for msg in messages {
//             match msg {
//                 Message::APIRequest(kv_cmd) => {
//                     match kv_cmd {
//                         KVCommand::Get(key) => {
//                             let leader = self.omni_paxos.get_current_leader().unwrap();
//                             let pid = self.pid;
//                             println!("Current leader: {:?} - {:?}\n", leader, pid);
//                             if leader != pid {
//                                 let msg = Message::APIResponse(APIResponse::NotALeader(leader));
//                                 self.network.send(0, msg).await;
//                             } else {
//                                 let value = self.database.handle_command(KVCommand::Get(key.clone()));
//                                 let msg = Message::APIResponse(APIResponse::Get(key, value));
//                                 self.network.send(0, msg).await;
//                             }
//                         },
//                         cmd => {
//                             self.omni_paxos.append(cmd).unwrap();
//                         },
//                     }
//                 }
//                 Message::OmniPaxosMsg(msg) => {
//                     self.omni_paxos.handle_incoming(msg);
//                 },
//                 _ => unimplemented!(),
//             }
//         }
//     }
// }

pub fn get_omni_paxos(pid: u64, nodes: Vec<u64>) -> Server {
    let server_config = ServerConfig {
        pid,
        election_tick_timeout: 5,
        ..Default::default()
    };
    let cluster_config = ClusterConfig {
        configuration_id: 1,
        nodes,
        ..Default::default()
    };
    let op_config = OmniPaxosConfig {
        server_config,
        cluster_config,
    };

    let my_path = "another_storage";
    let my_logopts = LogOptions::new(my_path);
    let mut my_sled_opts = Config::new();
    my_sled_opts = my_sled_opts.path(my_path);
    my_sled_opts = my_sled_opts.temporary(true);

// create configuration with given arguments
    let persistency_config = PersistentStorageConfig::with(my_path.into(), my_logopts, my_sled_opts);

    let omni_paxos = op_config
        .build(PersistentStorage::new(persistency_config))
        .expect("failed to build OmniPaxos");

     Server {
        omni_paxos,
        database: Database::new(),
        last_decided_idx: 0,
        pid,
    }


}