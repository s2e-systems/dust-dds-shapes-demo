use dust_dds::{
    domain::{
        domain_participant::DomainParticipant, domain_participant_factory::DomainParticipantFactory,
    },
    infrastructure::{
        listeners::NoOpListener,
        qos::{DataReaderQos, DataWriterQos, QosKind},
        qos_policy::{
            HistoryQosPolicy, HistoryQosPolicyKind, ReliabilityQosPolicy, ReliabilityQosPolicyKind,
        },
        status::NO_STATUS,
        time::DurationKind,
    },
    publication::{data_writer::DataWriter, publisher::Publisher},
    subscription::{
        data_reader::DataReader,
        sample_info::{ANY_INSTANCE_STATE, ANY_SAMPLE_STATE, ANY_VIEW_STATE},
        subscriber::Subscriber,
    },
};
use eframe::{
    egui,
    epaint::{vec2, Color32},
};
use std::sync::{Arc, Mutex};

use super::shapes_widget::{MovingShapeObject, ShapesWidget};

#[derive(Debug, Clone, dust_dds::topic_definition::type_support::DdsType)]
pub struct ShapeType {
    #[dust_dds(key)]
    pub color: String,
    pub x: i32,
    pub y: i32,
    pub shapesize: i32,
}

pub const PURPLE: Color32 = Color32::from_rgb(128, 0, 128);
pub const BLUE: Color32 = Color32::BLUE;
pub const RED: Color32 = Color32::RED;
pub const GREEN: Color32 = Color32::GREEN;
pub const YELLOW: Color32 = Color32::YELLOW;
pub const CYAN: Color32 = Color32::from_rgb(0, 255, 255);
pub const MAGENTA: Color32 = Color32::from_rgb(255, 0, 255);
pub const ORANGE: Color32 = Color32::from_rgb(255, 165, 0);

#[derive(Clone, Copy)]
pub enum ShapeKind {
    Circle,
    Triangle,
    Square,
}

impl ShapeKind {
    fn as_str(&self) -> &'_ str {
        match self {
            ShapeKind::Square => "Square",
            ShapeKind::Circle => "Circle",
            ShapeKind::Triangle => "Triangle",
        }
    }
}

struct ShapeWriter {
    writer: DataWriter<ShapeType>,
    shape: MovingShapeObject,
}
impl ShapeWriter {
    fn write(&self) {
        let data = (&self.shape).into();
        self.writer.write(&data, None).expect("writing failed");
    }
}

pub struct ShapesDemoApp {
    participant: DomainParticipant,
    publisher: Publisher,
    subscriber: Subscriber,
    reader_list: Vec<DataReader<ShapeType>>,
    shape_writer_list: Arc<Mutex<Vec<ShapeWriter>>>,
    window_open: Option<ShapeKind>,
    time: f64,
    is_reliable_writer: bool,
    is_reliable_reader: bool,
}

impl ShapesDemoApp {
    pub fn new() -> Self {
        let domain_id = 0;
        let participant_factory = DomainParticipantFactory::get_instance();
        let participant = participant_factory
            .create_participant(domain_id, QosKind::Default, NoOpListener::new(), NO_STATUS)
            .unwrap();
        let publisher = participant
            .create_publisher(QosKind::Default, NoOpListener::new(), NO_STATUS)
            .unwrap();
        let subscriber = participant
            .create_subscriber(QosKind::Default, NoOpListener::new(), NO_STATUS)
            .unwrap();

        let mut planner = periodic::Planner::new();

        let shape_writer_list = Arc::new(Mutex::new(Vec::<ShapeWriter>::new()));
        let shape_writer_list_clone = shape_writer_list.clone();
        planner.add(
            move || {
                for shape_writer in shape_writer_list_clone.lock().unwrap().iter() {
                    shape_writer.write()
                }
            },
            periodic::Every::new(std::time::Duration::from_millis(25)),
        );
        planner.start();

        Self {
            participant,
            publisher,
            subscriber,
            reader_list: vec![],
            shape_writer_list,
            window_open: None,
            time: 0.0,
            is_reliable_writer: true,
            is_reliable_reader: false,
        }
    }

    fn create_writer(&mut self, shape_kind: ShapeKind, color: &str, is_reliable: bool) {
        let topic_name = shape_kind.as_str();

        let topic = self
            .participant
            .create_topic::<ShapeType>(
                topic_name,
                "ShapeType",
                QosKind::Default,
                NoOpListener::new(),
                NO_STATUS,
            )
            .unwrap();
        let qos = if is_reliable {
            DataWriterQos {
                reliability: ReliabilityQosPolicy {
                    kind: ReliabilityQosPolicyKind::Reliable,
                    max_blocking_time: DurationKind::Infinite,
                },
                ..Default::default()
            }
        } else {
            DataWriterQos {
                reliability: ReliabilityQosPolicy {
                    kind: ReliabilityQosPolicyKind::BestEffort,
                    max_blocking_time: DurationKind::Infinite,
                },
                ..Default::default()
            }
        };
        let writer = self
            .publisher
            .create_datawriter(
                &topic,
                QosKind::Specific(qos),
                NoOpListener::new(),
                NO_STATUS,
            )
            .unwrap();

        let velocity = vec2(1.0, 1.0);
        let shape_type = &ShapeType {
            color: color.to_string(),
            x: 100,
            y: 80,
            shapesize: 30,
        };
        let shape = MovingShapeObject::from_shape_type(shape_kind, shape_type, velocity);

        let shape_writer = ShapeWriter { writer, shape };
        self.shape_writer_list.lock().unwrap().push(shape_writer);
    }

    fn create_reader(&mut self, topic_name: &str, is_reliable: bool) {
        let topic = self
            .participant
            .create_topic::<ShapeType>(
                topic_name,
                "ShapeType",
                QosKind::Default,
                NoOpListener::new(),
                NO_STATUS,
            )
            .unwrap();
        let qos = if is_reliable {
            DataReaderQos {
                reliability: ReliabilityQosPolicy {
                    kind: ReliabilityQosPolicyKind::Reliable,
                    max_blocking_time: DurationKind::Infinite,
                },
                history: HistoryQosPolicy {
                    kind: HistoryQosPolicyKind::KeepLast(1),
                },
                ..Default::default()
            }
        } else {
            DataReaderQos {
                reliability: ReliabilityQosPolicy {
                    kind: ReliabilityQosPolicyKind::BestEffort,
                    max_blocking_time: DurationKind::Infinite,
                },
                history: HistoryQosPolicy {
                    kind: HistoryQosPolicyKind::KeepLast(1),
                },
                ..Default::default()
            }
        };
        let reader = self
            .subscriber
            .create_datareader(
                &topic,
                QosKind::Specific(qos),
                NoOpListener::new(),
                NO_STATUS,
            )
            .unwrap();
        self.reader_list.push(reader);
    }

    fn read_data(&self, reader: &DataReader<ShapeType>) -> Vec<MovingShapeObject> {
        let topic_name = reader.get_topicdescription().unwrap().get_name().unwrap();
        let kind = match topic_name.as_str() {
            "Square" => ShapeKind::Square,
            "Circle" => ShapeKind::Circle,
            "Triangle" => ShapeKind::Triangle,
            _ => panic!("Unsupported shape"),
        };

        let mut shapes = vec![];
        let mut previous_handle = None;
        while let Ok(samples) = reader.read_next_instance(
            1,
            previous_handle,
            ANY_SAMPLE_STATE,
            ANY_VIEW_STATE,
            ANY_INSTANCE_STATE,
        ) {
            if let Some(sample) = samples.first() {
                previous_handle = Some(sample.sample_info().instance_handle);
                if let Ok(shape_type) = sample.data() {
                    let shape =
                        MovingShapeObject::from_shape_type(kind, &shape_type, egui::Vec2::ZERO);
                    shapes.push(shape);
                }
            }
        }
        shapes
    }

    fn menu_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Publish");
        if ui.button(ShapeKind::Square.as_str()).clicked() {
            self.window_open = Some(ShapeKind::Square);
        };
        if ui.button(ShapeKind::Circle.as_str()).clicked() {
            self.window_open = Some(ShapeKind::Circle);
        };
        if ui.button(ShapeKind::Triangle.as_str()).clicked() {
            self.window_open = Some(ShapeKind::Triangle);
        };
        ui.separator();
        ui.heading("Subscribe");
        if ui.button(ShapeKind::Square.as_str()).clicked() {
            self.create_reader(ShapeKind::Square.as_str(), self.is_reliable_reader)
        };
        if ui.button(ShapeKind::Circle.as_str()).clicked() {
            self.create_reader(ShapeKind::Circle.as_str(), self.is_reliable_reader)
        };
        if ui.button(ShapeKind::Triangle.as_str()).clicked() {
            self.create_reader(ShapeKind::Triangle.as_str(), self.is_reliable_reader)
        };
        ui.checkbox(&mut self.is_reliable_reader, "reliable");
    }
}

impl eframe::App for ShapesDemoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(shape_kind) = self.window_open {
            egui::Window::new("Publish").show(ctx, |ui| {
                if ui.button("PURPLE").clicked() {
                    self.window_open = None;
                    self.create_writer(shape_kind, "PURPLE", self.is_reliable_writer);
                }
                if ui.button("BLUE").clicked() {
                    self.window_open = None;
                    self.create_writer(shape_kind, "BLUE", self.is_reliable_writer);
                }
                if ui.button("RED").clicked() {
                    self.window_open = None;
                    self.create_writer(shape_kind, "RED", self.is_reliable_writer);
                }
                if ui.button("GREEN").clicked() {
                    self.window_open = None;
                    self.create_writer(shape_kind, "GREEN", self.is_reliable_writer);
                }
                if ui.button("YELLOW").clicked() {
                    self.window_open = None;
                    self.create_writer(shape_kind, "YELLOW", self.is_reliable_writer);
                }
                if ui.button("CYAN").clicked() {
                    self.window_open = None;
                    self.create_writer(shape_kind, "CYAN", self.is_reliable_writer);
                }
                if ui.button("MAGENTA").clicked() {
                    self.window_open = None;
                    self.create_writer(shape_kind, "MAGENTA", self.is_reliable_writer);
                }
                if ui.button("ORANGE").clicked() {
                    self.window_open = None;
                    self.create_writer(shape_kind, "ORANGE", self.is_reliable_writer);
                }
                ui.checkbox(&mut self.is_reliable_writer, "reliable");
            });
        }
        let is_landscape = ctx.screen_rect().aspect_ratio() > 1.0;

        if is_landscape {
            egui::SidePanel::left("menu_panel")
                .default_width(100.0)
                .show(ctx, |ui| self.menu_panel(ui));
        } else {
            egui::TopBottomPanel::top("menu_panel").show(ctx, |ui| self.menu_panel(ui));
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let rect_size = egui::vec2(235.0, 265.0);

            let mut shape_list = Vec::new();
            for reader in &self.reader_list {
                for shape in self.read_data(reader) {
                    shape_list.push(shape);
                }
            }

            let time = ui.input(|i| i.time);
            let time_delta = (time - self.time) as f32;
            self.time = time;
            for writer in self.shape_writer_list.lock().unwrap().iter_mut() {
                writer.shape.move_within_rect(rect_size, time_delta * 40.0);
                shape_list.push(writer.shape.clone());
            }
            ui.add(ShapesWidget::new(rect_size, shape_list.as_slice()));
            ctx.request_repaint_after(std::time::Duration::from_millis(40));
        });
    }
}
