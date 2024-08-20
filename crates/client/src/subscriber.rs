use std::sync::Arc;

use eframe::egui::{self, text::LayoutJob, Color32, Id, Sense, TextFormat, Vec2};
use parking_lot::Mutex;
use time::{format_description, OffsetDateTime};
use tracing::{
    field::{Field, Visit},
    span, Event, Level, Subscriber,
};
use tracing_subscriber::{
    layer::Context,
    registry::{LookupSpan, Scope, SpanRef},
    Layer,
};

#[derive(Clone)]
pub struct EguiLayer {
    events: Arc<Mutex<Vec<(EventData, ScopeData)>>>,
    level: Level,
}

impl Default for EguiLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl EguiLayer {
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
            level: Level::INFO,
        }
    }

    pub fn with_level(mut self, level: Level) -> Self {
        self.level = level;
        self
    }

    pub fn add_event_with_scope(&self, event: EventData, scope: ScopeData) {
        if event.level <= self.level {
            self.events.lock().push((event, scope));
        }
    }

    pub fn level_color(level: Level) -> Color32 {
        match level {
            Level::TRACE => Color32::GRAY,
            Level::DEBUG => Color32::LIGHT_BLUE,
            Level::WARN => Color32::from_rgb(196, 160, 0),
            Level::INFO => Color32::LIGHT_GREEN,
            Level::ERROR => Color32::LIGHT_RED,
        }
    }

    pub fn ui(&self, ui: &mut egui::Ui) {
        let lock = self.events.lock();
        for (event, scope) in lock.iter() {
            let color = Self::level_color(event.level);

            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), Id::new("log_data").with(event.time), false)
                .show_header(ui, |ui| {
                    ui.colored_label(color, format!("{}", event.level));
                    ui.horizontal(|ui| {
                        ui.colored_label(color, format!("{}:", &event.target));
                        if let Some((_, content)) = event.fields.0.iter().find(|(name, _)| name == "message") {
                            ui.colored_label(color, content);
                        }
                        for (name, content) in &event.fields.0 {
                            if name == "message" {
                                continue;
                            }

                            ui.colored_label(color, format!("{name}: {content}"));
                        }
                    });
                })
                .body(|ui| {
                    ui.vertical(|ui| {
                        ui.label(format!(
                            "Time: {} {} {}",
                            event.time.date(),
                            event
                                .time
                                .time()
                                .format(&format_description::parse("[hour]:[minute]:[second]").unwrap())
                                .unwrap(),
                            event.time.offset()
                        ));

                        let draw_layout_job = |ui: &mut egui::Ui, job| {
                            let galley = ui.fonts(|fonts| fonts.layout_job(job));
                            let response = ui.allocate_response(galley.size(), Sense::hover());
                            // response.rect = response.rect.translate(Vec2::new(25.0, 0.0));

                            ui.painter().galley(response.rect.left_top(), galley, Color32::WHITE);
                        };

                        if let Some((file, line)) = event.file.as_ref().and_then(|f| event.line.map(|l| (f, l))) {
                            let mut job = LayoutJob::default();

                            job.append(
                                "at",
                                5.0,
                                TextFormat {
                                    italics: true,
                                    ..Default::default()
                                },
                            );

                            job.append(&format!("{}:{}", file, line), 5.0, TextFormat::default());

                            draw_layout_job(ui, job);
                        }

                        for span in &scope.spans {
                            let mut job = LayoutJob::default();
                            job.append(
                                "in",
                                5.0,
                                TextFormat {
                                    italics: true,
                                    ..Default::default()
                                },
                            );

                            job.append(span.name, 5.0, TextFormat::default());

                            if !span.fields.0.is_empty() {
                                job.append(
                                    "with",
                                    5.0,
                                    TextFormat {
                                        italics: true,
                                        ..Default::default()
                                    },
                                );
                            }

                            for (name, content) in &span.fields.0 {
                                job.append(&format!("{name}: {content}"), 5.0, TextFormat::default());
                            }

                            draw_layout_job(ui, job);
                        }
                    });
                });
        }
    }
}

impl<S> Layer<S> for EguiLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(&self, attrs: &span::Attributes<'_>, id: &span::Id, ctx: Context<'_, S>) {
        let mut fields = Fields::default();
        attrs.record(&mut FieldsVisitor(&mut fields));

        if let Some(span) = ctx.span(id) {
            span.extensions_mut().insert(SpanFieldsExtension { fields });
        }
    }

    fn on_record(&self, span: &span::Id, values: &span::Record<'_>, ctx: Context<'_, S>) {
        let Some(span) = ctx.span(span) else {
            return;
        };

        let mut fields = Fields::default();
        values.record(&mut FieldsVisitor(&mut fields));

        match span.extensions_mut().get_mut::<SpanFieldsExtension>() {
            Some(span_fields) => {
                span_fields.fields.0.extend(fields.0);
            }
            None => span.extensions_mut().insert(SpanFieldsExtension { fields }),
        };
    }

    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        self.add_event_with_scope(EventData::new(event), ScopeData::new(ctx.event_scope(event)));
    }
}

#[derive(Debug, Clone)]
pub struct EventData {
    pub target: String,
    pub level: tracing::Level,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub fields: Fields,
    pub time: OffsetDateTime,
}

impl EventData {
    pub fn new(event: &Event<'_>) -> Self {
        let metadata = event.metadata();

        let mut fields = Fields::default();
        event.record(&mut FieldsVisitor(&mut fields));

        let time = OffsetDateTime::now_local().unwrap_or(OffsetDateTime::now_utc());

        EventData {
            target: metadata.target().to_owned(),
            level: metadata.level().to_owned(),
            file: metadata.file().map(ToString::to_string),
            line: metadata.line(),
            fields,
            time,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Fields(Vec<(String, String)>);

struct FieldsVisitor<'a>(&'a mut Fields);

impl<'a> Visit for FieldsVisitor<'a> {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.0 .0.push((field.name().to_string(), format!("{:?}", value)));
    }
}

pub struct SpanFieldsExtension {
    fields: Fields,
}

pub struct ScopeData {
    spans: Vec<SpanData>,
}

impl ScopeData {
    pub fn new<S>(scope: Option<Scope<'_, S>>) -> Self
    where
        S: Subscriber + for<'a> LookupSpan<'a>,
    {
        let spans = scope.map(|scope| scope.from_root().map(SpanData::new).collect()).unwrap_or_default();

        Self { spans }
    }
}

pub struct SpanData {
    pub name: &'static str,
    pub fields: Fields,
}

impl SpanData {
    pub fn new<S>(span: SpanRef<'_, S>) -> Self
    where
        S: Subscriber + for<'a> LookupSpan<'a>,
    {
        let fields = span
            .extensions()
            .get::<SpanFieldsExtension>()
            .map(|ext| &ext.fields)
            .cloned()
            .unwrap_or_default();

        Self { name: span.name(), fields }
    }
}
