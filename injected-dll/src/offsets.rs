use std::{collections::HashMap, sync::RwLock};

use serde::{de::Error, Deserialize, Deserializer};

const DEFAULT_OFFSET: isize = 0x180000000;

lazy_static! {
    pub static ref OFFSETS: Metadata =
        serde_json::from_str::<'_, Root>(include_str!("../../metadata.json"))
            .unwrap()
            .address_map;
    static ref METHOD_DEFINITIONS_BY_NAME: RwLock<HashMap<String, isize>> = RwLock::new(
        OFFSETS
            .method_definitions
            .iter()
            .map(|m| (m.name.clone(), m.virtual_address))
            .collect()
    );
    static ref APIS_BY_NAME: RwLock<HashMap<String, isize>> = RwLock::new(
        OFFSETS
            .apis
            .iter()
            .map(|m| (m.name.clone(), m.virtual_address))
            .collect()
    );
}

fn from_hex<'de, D>(deserializer: D) -> Result<isize, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    if !s.starts_with("0x") {
        return Err(D::Error::custom("Hex number doesn't start with '0x'"));
    }
    isize::from_str_radix(&s[2..], 16).map_err(D::Error::custom)
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub address_map: Metadata,
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub method_definitions: Vec<MethodDefinition>,
    // pub constructed_generic_methods: Vec<ConstructedGenericMethod>,
    // pub custom_attributes_generators: Vec<CustomAttributesGenerator>,
    // pub method_invokers: Vec<MethodInvoker>,
    // pub string_literals: Vec<StringLiteral>,
    // pub type_info_pointers: Vec<TypeInfoPointer>,
    // pub type_ref_pointers: Vec<TypeRefPointer>,
    // pub method_info_pointers: Vec<MethodInfoPointer>,
    // pub function_addresses: Vec<String>,
    // pub type_metadata: Vec<TypeMetadaum>,
    // pub function_metadata: Vec<FunctionMetadaum>,
    // pub array_metadata: Vec<ArrayMetadaum>,
    pub apis: Vec<Api>,
    // pub exports: Vec<Export>,
    // pub symbols: Vec<Value>,
}

impl Metadata {
    pub fn method(&self, name: &str) -> isize {
        let methods_by_name = METHOD_DEFINITIONS_BY_NAME.read().unwrap();
        methods_by_name[name] - DEFAULT_OFFSET
    }

    pub fn api(&self, name: &str) -> isize {
        let api_by_name = APIS_BY_NAME.read().unwrap();
        // Where does the magic 0xc00 offset come from???????????
        api_by_name[name] - DEFAULT_OFFSET - 0xc00
    }
}

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MethodDefinition {
    #[serde(deserialize_with = "from_hex")]
    pub virtual_address: isize,
    pub name: String,
    pub signature: String,
    pub dot_net_signature: String,
}

// #[derive(Default, Debug, Clone, PartialEq, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct ConstructedGenericMethod {
//     pub virtual_address: String,
//     pub name: String,
//     pub signature: String,
//     pub dot_net_signature: String,
// }

// #[derive(Default, Debug, Clone, PartialEq, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct CustomAttributesGenerator {
//     pub virtual_address: String,
//     pub name: String,
//     pub signature: String,
// }

// #[derive(Default, Debug, Clone, PartialEq, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct MethodInvoker {
//     pub virtual_address: String,
//     pub name: String,
//     pub signature: String,
// }

// #[derive(Default, Debug, Clone, PartialEq, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct StringLiteral {
//     pub virtual_address: String,
//     pub name: String,
//     pub string: String,
// }

// #[derive(Default, Debug, Clone, PartialEq, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct TypeInfoPointer {
//     pub virtual_address: String,
//     pub name: String,
//     #[serde(rename = "type")]
//     pub type_field: String,
//     pub dot_net_type: String,
// }

// #[derive(Default, Debug, Clone, PartialEq, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct TypeRefPointer {
//     pub virtual_address: String,
//     pub name: String,
//     pub dot_net_type: String,
// }

// #[derive(Default, Debug, Clone, PartialEq, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct MethodInfoPointer {
//     pub virtual_address: String,
//     pub name: String,
//     pub dot_net_signature: String,
// }

// #[derive(Default, Debug, Clone, PartialEq, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct TypeMetadaum {
//     pub virtual_address: String,
//     pub name: String,
//     #[serde(rename = "type")]
//     pub type_field: String,
// }

// #[derive(Default, Debug, Clone, PartialEq, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct FunctionMetadaum {
//     pub virtual_address: String,
//     pub name: String,
//     pub signature: String,
// }

// #[derive(Default, Debug, Clone, PartialEq, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct ArrayMetadaum {
//     pub virtual_address: String,
//     pub name: String,
//     #[serde(rename = "type")]
//     pub type_field: String,
//     pub count: i64,
// }

#[derive(Default, Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Api {
    #[serde(deserialize_with = "from_hex")]
    pub virtual_address: isize,
    pub name: String,
    pub signature: String,
}

// #[derive(Default, Debug, Clone, PartialEq, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct Export {
//     pub virtual_address: String,
//     pub name: String,
// }
