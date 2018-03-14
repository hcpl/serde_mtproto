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
extern crate serde_yaml;
extern crate toml;


use serde_bytes::ByteBuf;


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, MtProtoIdentifiable, MtProtoSized)]
#[id = "0x7e298afc"]
struct Data {
    id: u64,
    raw_data: ByteBuf,
    metadata: Metadata,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, MtProtoIdentifiable, MtProtoSized)]
#[id = "0xb3185db0"]
struct Metadata {
    username: String,
    encrypted: bool,
    seen_times: u32,
}


lazy_static! {
    static ref DATA: Data = Data {
        id: 0x5922_0494_9ed2_18af,
        raw_data: ByteBuf::from(b"content".as_ref().to_owned()),
        metadata: Metadata {
            username: "new_user".to_owned(),
            encrypted: false,
            seen_times: 31,
        },
    };

    static ref DATA_MTPROTO: Vec<u8> = serde_mtproto::to_bytes(&*DATA).unwrap();
    static ref DATA_JSON: Vec<u8> = serde_json::to_vec(&*DATA).unwrap();
    static ref DATA_YAML: Vec<u8> = serde_yaml::to_vec(&*DATA).unwrap();
    static ref DATA_TOML: Vec<u8> = toml::to_vec(&*DATA).unwrap();
}


#[test]
fn json_to_mtproto() {
    let data: Data = serde_json::from_slice(&*DATA_JSON).unwrap();
    let data_mtproto = serde_mtproto::to_bytes(&data).unwrap();

    assert_eq!(data_mtproto, *DATA_MTPROTO);
}

#[test]
fn mtproto_to_json() {
    let data: Data = serde_mtproto::from_bytes(&*DATA_MTPROTO, &[]).unwrap();
    let data_json = serde_json::to_vec(&data).unwrap();

    assert_eq!(data_json, *DATA_JSON);
}

#[test]
fn yaml_to_mtproto() {
    let data: Data = serde_yaml::from_slice(&*DATA_YAML).unwrap();
    let data_mtproto = serde_mtproto::to_bytes(&data).unwrap();

    assert_eq!(data_mtproto, *DATA_MTPROTO);
}

#[test]
fn mtproto_to_yaml() {
    let data: Data = serde_mtproto::from_bytes(&*DATA_MTPROTO, &[]).unwrap();
    let data_yaml = serde_yaml::to_vec(&data).unwrap();

    assert_eq!(data_yaml, *DATA_YAML);
}

#[test]
fn toml_to_mtproto() {
    let data: Data = toml::from_slice(&*DATA_TOML).unwrap();
    let data_mtproto = serde_mtproto::to_bytes(&data).unwrap();

    assert_eq!(data_mtproto, *DATA_MTPROTO);
}

#[test]
fn mtproto_to_toml() {
    let data: Data = serde_mtproto::from_bytes(&*DATA_MTPROTO, &[]).unwrap();
    let data_toml = toml::to_vec(&data).unwrap();

    assert_eq!(data_toml, *DATA_TOML);
}


#[test]
fn extern_serde_formats_interop() {
    // JSON & YAML
    {
        let data: Data = serde_yaml::from_slice(&*DATA_YAML).unwrap();
        let data_json = serde_json::to_vec(&data).unwrap();

        assert_eq!(data_json, *DATA_JSON);

        let data: Data = serde_json::from_slice(&*DATA_JSON).unwrap();
        let data_yaml = serde_yaml::to_vec(&data).unwrap();

        assert_eq!(data_yaml, *DATA_YAML);
    }

    // JSON & TOML
    {
        let data: Data = toml::from_slice(&*DATA_TOML).unwrap();
        let data_json = serde_json::to_vec(&data).unwrap();

        assert_eq!(data_json, *DATA_JSON);

        let data: Data = serde_json::from_slice(&*DATA_JSON).unwrap();
        let data_toml = toml::to_vec(&data).unwrap();

        assert_eq!(data_toml, *DATA_TOML);
    }

    // YAML & TOML
    {
        let data: Data = toml::from_slice(&*DATA_TOML).unwrap();
        let data_yaml = serde_yaml::to_vec(&data).unwrap();

        assert_eq!(data_yaml, *DATA_YAML);

        let data: Data = serde_yaml::from_slice(&*DATA_YAML).unwrap();
        let data_toml = toml::to_vec(&data).unwrap();

        assert_eq!(data_toml, *DATA_TOML);
    }
}
