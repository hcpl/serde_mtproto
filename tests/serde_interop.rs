#[macro_use]
extern crate lazy_static;
extern crate serde;
extern crate serde_bytes;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde_mtproto;
#[macro_use]
extern crate serde_mtproto_derive;


use serde_bytes::ByteBuf;


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, MtProtoIdentifiable, MtProtoSized)]
#[id = "7e298afc"]
struct Data {
    id: u64,
    metadata: Metadata,
    raw_data: ByteBuf,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, MtProtoIdentifiable, MtProtoSized)]
#[id = "b3185db0"]
struct Metadata {
    username: String,
    encrypted: bool,
    seen_times: u32,
}


lazy_static! {
    static ref DATA: Data = Data {
        id: 6422701054949988527,
        metadata: Metadata {
            username: "new_user".to_owned(),
            encrypted: false,
            seen_times: 31,
        },
        raw_data: ByteBuf::from(b"content".as_ref().to_owned()),
    };

    static ref DATA_MTPROTO: Vec<u8> = serde_mtproto::to_bytes(&*DATA).unwrap();
    static ref DATA_JSON: Vec<u8> = serde_json::to_vec(&*DATA).unwrap();
}


#[test]
fn json_to_mtproto() {
    let data: Data = serde_json::from_slice(&*DATA_JSON).unwrap();
    let data_mtproto = serde_mtproto::to_bytes(&data).unwrap();

    assert_eq!(data_mtproto, *DATA_MTPROTO);
}

#[test]
fn mtproto_to_json() {
    let data: Data = serde_mtproto::from_bytes(&*DATA_MTPROTO, None).unwrap();
    let data_json = serde_json::to_vec(&data).unwrap();

    assert_eq!(data_json, *DATA_JSON);
}
