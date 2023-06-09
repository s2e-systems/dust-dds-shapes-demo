#[derive(Debug, serde::Deserialize, serde::Serialize, dust_dds::topic_definition::type_support::DdsSerde, dust_dds::topic_definition::type_support::DdsType)]
pub struct ShapeType {
    #[key] pub color: String,
    pub x: i32,
    pub y: i32,
    pub shapesize: i32,
}
