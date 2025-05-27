use hydro_lang::*;
use akd::{AkdLabel, AkdValue};
use serde::{Serialize, Deserialize};


#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct Label(pub Vec<u8>);
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Value(pub Vec<u8>);

impl From<AkdLabel> for Label { fn from(l: AkdLabel) -> Self { Label(l.0) } }
impl From<AkdValue> for Value { fn from(v: AkdValue) -> Self { Value(v.0) } }
impl From<Label> for AkdLabel { fn from(l: Label) -> Self { AkdLabel(l.0) } }
impl From<Value> for AkdValue { fn from(v: Value) -> Self { AkdValue(v.0) } }

/* ---- 2. messages now use the wrappers ---- */
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PublishMsg {
    pub updates: Vec<(Label, Value)>,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PublishMsgResponse {
    pub epoch: u64,
    pub root_hash: Vec<u8>,
}

pub struct Server {}

pub struct Clients {}

// API
    // Publish: publish client ID, public key
    // - label: AkdLabel (client identifier)
    // - value: AkdValue (public key data)
    // - epoch: u64 (optional version number)

    // Lookup: verify mapping
    // - label: AkdLabel (client identifier to lookup)
    // - epoch: u64 (specific version, or latest if not specified)
    // - Returns: LookupProof for verification

    // History: verify history of mappings
    // - label: AkdLabel (client identifier)
    // - history_params: HistoryParams (epoch range, limit)
    // - Returns: Vec<HistoryProof> with historical values

pub fn akd_server<'a>(flow: &FlowBuilder<'a>) -> (Process<'a, Server>, Cluster<'a, Clients>) {
    // generate clients and commands
    let clients = flow.cluster::<Clients>();
    // Assume single server.
    let server = flow.process::<Server>();

    let client_requests = clients
        .source_iter(q!([PublishMsg {
            updates: vec![(
                AkdLabel::from("first entry").into(),
                AkdValue::from("first value").into(),
            )],
        }]))
        .send_bincode(&server)
        .inspect(q!(|(id, msg)| println!(
            "...publishing entry {:?} from client #{}...",
            msg.updates, id
        )));
    
    // Add a simple response from server back to clients
    client_requests
        .map(q!(|(id, msg)| (id, PublishMsgResponse {
            epoch: 1,
            root_hash: vec![0, 1, 2, 3] // Dummy hash for now
        })))
        .broadcast_bincode(&clients)
        .for_each(q!(|(id, resp)| println!(
            "Client #{} received response: epoch={}, hash={:?}",
            id, resp.epoch, resp.root_hash
        )));

    (server, clients)
}

