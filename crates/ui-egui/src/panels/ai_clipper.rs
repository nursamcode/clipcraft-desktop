//! AI Clipper Panel (`[DESKTOP-03]`): Autonomous YouTube Video to Viral 9:16 Shorts converter.
//! Connects directly to `clipcraft-api` (`https://clipper.ezuuk.my.id`) for AI ingestion,
//! speech-to-text, 9Router LLM curation, face tracking, and 9:16 timeline generation.

use egui::{Color32, FontId, Rect, Rounding, Stroke, Vec2, pos2, vec2};
use serde_json::json;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::FilmcraftApp;
use crate::panels::panel_state::AiClipItem;

/// Global channel for background network requests between UI thread and HTTP workers.
static WORKER_CHANNEL: std::sync::OnceLock<(Sender<WorkerMsg>, Arc<Mutex<Receiver<WorkerResponse>>>)> =
    std::sync::OnceLock::new();

enum WorkerMsg {
    StartJob {
        endpoint: String,
        url: String,
        genre: String,
        min_score: i32,
    },
    PollJob {
        endpoint: String,
        job_id: String,
    },
    ListenSse {
        endpoint: String,
        job_id: String,
    },
}

enum WorkerResponse {
    JobCreated {
        job_id: String,
    },
    JobStatus {
        job_id: String,
        status: String,
        progress: i32,
        current_step: String,
        error: Option<String>,
        clips: Vec<AiClipItem>,
    },
    Error(String),
}

fn get_worker() -> (&'static Sender<WorkerMsg>, Arc<Mutex<Receiver<WorkerResponse>>>) {
    let (tx, rx_shared) = WORKER_CHANNEL.get_or_init(|| {
        let (req_tx, req_rx) = channel::<WorkerMsg>();
        let (resp_tx, resp_rx) = channel::<WorkerResponse>();

        // Spawn background worker thread
        thread::spawn(move || {
            while let Ok(msg) = req_rx.recv() {
                match msg {
                    WorkerMsg::StartJob {
                        endpoint,
                        url,
                        genre,
                        min_score,
                    } => {
                        let client = ureq::builder()
                            .timeout(std::time::Duration::from_secs(15))
                            .build();
                        let api_url = format!("{}/api/v1/jobs", endpoint.trim_end_matches('/'));
                        let payload = json!({
                            "source_url": url,
                            "content_genre": genre,
                            "min_virality_score": min_score
                        });

                        match client.post(&api_url).send_json(payload) {
                            Ok(res) => {
                                let mut reader = res.into_body();
                                if let Ok(val) = serde_json::from_reader::<_, serde_json::Value>(reader.as_reader()) {
                                    if let Some(jid) = val["id"].as_str() {
                                        let jid_str = jid.to_string();
                                        let _ = resp_tx.send(WorkerResponse::JobCreated {
                                            job_id: jid_str.clone(),
                                        });
                                        let sse_endpoint = endpoint.clone();
                                        let resp_tx_sse = resp_tx.clone();
                                        thread::spawn(move || {
                                            let sse_url = format!("{}/api/v1/jobs/{}/events", sse_endpoint.trim_end_matches('/'), jid_str);
                                            if let Ok(resp) = ureq::get(&sse_url).call() {
                                                use std::io::BufRead;
                                                let reader = std::io::BufReader::new(resp.into_body().as_reader());
                                                for line in reader.lines().map_while(Result::ok) {
                                                    if line.starts_with("data: ") {
                                                        let json_str = &line["data: ".len()..];
                                                        if let Ok(data) = serde_json::from_str::<serde_json::Value>(json_str) {
                                                            let st = data["status"].as_str().unwrap_or("").to_string();
                                                            let pr = data["progress"].as_i64().unwrap_or(0) as i32;
                                                            let sp = data["step"].as_str().unwrap_or("").to_string();
                                                            let er = data["error"].as_str().map(|s| s.to_string());
                                                            let _ = resp_tx_sse.send(WorkerResponse::JobStatus {
                                                                job_id: jid_str.clone(),
                                                                status: st.clone(),
                                                                progress: pr,
                                                                current_step: sp,
                                                                error: er,
                                                                clips: Vec::new(),
                                                            });
                                                            if matches!(st.as_str(), "COMPLETED" | "FAILED") {
                                                                break;
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        });
                                    }
                                }
                            }
                            Err(e) => {
                                let _ = resp_tx.send(WorkerResponse::Error(format!(
                                    "Gagal memulai job: {e}"
                                )));
                            }
                        }
                    }
                    WorkerMsg::PollJob { endpoint, job_id } => {
                        let client = ureq::builder()
                            .timeout(std::time::Duration::from_secs(10))
                            .build();
                        let api_url = format!(
                            "{}/api/v1/jobs/{}",
                            endpoint.trim_end_matches('/'),
                            job_id
                        );

                        match client.get(&api_url).call() {
                            Ok(res) => {
                                let mut reader = res.into_body();
                                if let Ok(val) = serde_json::from_reader::<_, serde_json::Value>(reader.as_reader()) {
                                    let status = val["status"].as_str().unwrap_or("UNKNOWN").to_string();
                                    let progress = val["progress"].as_i64().unwrap_or(0) as i32;
                                    let step = val["current_step"].as_str().unwrap_or("").to_string();
                                    let err = val["error_message"].as_str().map(|s| s.to_string());

                                    let mut clips = Vec::new();
                                    if let Some(raw_clips) = val["clips"].as_array() {
                                        for c in raw_clips {
                                            clips.push(AiClipItem {
                                                id: c["id"].as_str().unwrap_or("").to_string(),
                                                title: c["title"].as_str().unwrap_or("").to_string(),
                                                reason: c["reason"].as_str().unwrap_or("").to_string(),
                                                description: c["description"].as_str().unwrap_or("").to_string(),
                                                tags: c["tags"].as_str().unwrap_or("").to_string(),
                                                virality_score: c["virality_score"].as_i64().unwrap_or(0) as i32,
                                                start_sec: c["start_sec"].as_f64().unwrap_or(0.0),
                                                end_sec: c["end_sec"].as_f64().unwrap_or(0.0),
                                                duration_sec: c["duration_sec"].as_f64().unwrap_or(0.0),
                                                layout_type: c["layout_type"].as_str().unwrap_or("solo").to_string(),
                                                video_url: c["video_url"].as_str().unwrap_or("").to_string(),
                                            });
                                        }
                                    }

                                    let _ = resp_tx.send(WorkerResponse::JobStatus {
                                        job_id,
                                        status,
                                        progress,
                                        current_step: step,
                                        error: err,
                                        clips,
                                    });
                                }
                            }
                            Err(e) => {
                                let _ = resp_tx.send(WorkerResponse::Error(format!(
                                    "Poll error: {e}"
                                )));
                            }
                        }
                    }
                }
            }
        });

        (req_tx, Arc::new(Mutex::new(resp_rx)))
    });

    (tx, Arc::clone(rx_shared))
}

pub fn show(app: &mut FilmcraftApp, ui: &mut egui::Ui, rect: Rect) {
    let t = app.tokens;
    let (tx, rx_mutex) = get_worker();

    // Process incoming background responses
    if let Ok(rx) = rx_mutex.try_lock() {
        while let Ok(resp) = rx.try_recv() {
            match resp {
                WorkerResponse::JobCreated { job_id } => {
                    app.ui.panels.ai_clipper.active_job_id = Some(job_id.clone());
                    app.ui.panels.ai_clipper.status = "QUEUED".to_string();
                    app.ui.panels.ai_clipper.progress = 0;
                    app.ui.panels.ai_clipper.current_step = "Menunggu antrean server...".to_string();
                    app.ui.panels.ai_clipper.error_message = None;
                    app.ui.panels.ai_clipper.clips.clear();
                }
                WorkerResponse::JobStatus {
                    job_id,
                    status,
                    progress,
                    current_step,
                    error,
                    clips,
                } => {
                    if app.ui.panels.ai_clipper.active_job_id.as_deref() == Some(&job_id) {
                        app.ui.panels.ai_clipper.status = status;
                        app.ui.panels.ai_clipper.progress = progress;
                        app.ui.panels.ai_clipper.current_step = current_step;
                        app.ui.panels.ai_clipper.error_message = error;
                        if !clips.is_empty() {
                            app.ui.panels.ai_clipper.clips = clips;
                        }
                    }
                }
                WorkerResponse::Error(err_msg) => {
                    app.ui.panels.ai_clipper.error_message = Some(err_msg);
                }
            }
        }
    }

    // Trigger poll periodically if job is active
    let is_busy = matches!(
        app.ui.panels.ai_clipper.status.as_str(),
        "QUEUED" | "DOWNLOADING" | "TRANSCRIBING" | "CURATING" | "RENDERING"
    );
    if is_busy {
        if let Some(ref jid) = app.ui.panels.ai_clipper.active_job_id {
            // Request repaint for smooth UI progress
            ui.ctx().request_repaint_after(std::time::Duration::from_millis(800));
            let _ = tx.send(WorkerMsg::PollJob {
                endpoint: app.ui.panels.ai_clipper.api_endpoint.clone(),
                job_id: jid.clone(),
            });
        }
    }

    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect.shrink(8.0)));
    let ui = &mut child;

    egui::ScrollArea::vertical()
        .id_salt("ai_clipper_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 10.0);

            // Title Banner
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("⚡ AI Clipper Studio")
                        .font(FontId::proportional(17.0))
                        .strong()
                        .color(Color32::from_rgb(167, 139, 250)),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let (status_text, status_col) = if is_busy {
                        ("● Processing", Color32::from_rgb(250, 204, 21))
                    } else {
                        ("● Ready", Color32::from_rgb(52, 211, 153))
                    };
                    ui.label(egui::RichText::new(status_text).color(status_col).small());
                });
            });

            ui.separator();

            // Form Section
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("YouTube Video URL").small().color(t.text_dim));
                let resp = ui.add(
                    egui::TextEdit::singleline(&mut app.ui.panels.ai_clipper.source_url)
                        .hint_text("https://www.youtube.com/watch?v=...")
                        .desired_width(f32::INFINITY),
                );
                app.auto.add("ai_clipper.url", resp.rect, "Source URL");

                ui.add_space(4.0);

                // Genre and Min Score
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new("Gaya Konten").small().color(t.text_dim));
                        egui::ComboBox::from_id_salt("ai_clipper_genre")
                            .selected_text(match app.ui.panels.ai_clipper.content_genre.as_str() {
                                "duo" => "Podcast Duo (Stacked)",
                                "solo" => "Solo Creator / Edukasi",
                                "game" => "Gaming Streamer",
                                _ => "Multi-Person Wide",
                            })
                            .width(180.0)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut app.ui.panels.ai_clipper.content_genre,
                                    "duo".to_string(),
                                    "Podcast Duo (Stacked)",
                                );
                                ui.selectable_value(
                                    &mut app.ui.panels.ai_clipper.content_genre,
                                    "solo".to_string(),
                                    "Solo Creator / Edukasi",
                                );
                                ui.selectable_value(
                                    &mut app.ui.panels.ai_clipper.content_genre,
                                    "game".to_string(),
                                    "Gaming Streamer",
                                );
                                ui.selectable_value(
                                    &mut app.ui.panels.ai_clipper.content_genre,
                                    "multi".to_string(),
                                    "Multi-Person Wide",
                                );
                            });
                    });

                    ui.add_space(12.0);

                    ui.vertical(|ui| {
                        ui.label(
                            egui::RichText::new(format!(
                                "Min Virality: {}",
                                app.ui.panels.ai_clipper.min_virality_score
                            ))
                            .small()
                            .color(t.text_dim),
                        );
                        ui.add(
                            egui::Slider::new(
                                &mut app.ui.panels.ai_clipper.min_virality_score,
                                60..=95,
                            )
                            .show_value(false),
                        );
                    });
                });

                ui.add_space(6.0);

                // Action Button
                let can_submit = !is_busy && !app.ui.panels.ai_clipper.source_url.trim().is_empty();
                let btn = egui::Button::new(
                    egui::RichText::new(if is_busy {
                        "Sedang Memproses Klip..."
                    } else {
                        "⚡ Auto-Generate Shorts 9:16"
                    })
                    .font(FontId::proportional(14.0))
                    .strong()
                    .color(Color32::WHITE),
                )
                .fill(if can_submit {
                    Color32::from_rgb(124, 58, 237)
                } else {
                    Color32::from_rgb(60, 60, 70)
                })
                .min_size(vec2(ui.available_width(), 36.0));

                if ui.add_enabled(can_submit, btn).clicked() {
                    let _ = tx.send(WorkerMsg::StartJob {
                        endpoint: app.ui.panels.ai_clipper.api_endpoint.clone(),
                        url: app.ui.panels.ai_clipper.source_url.trim().to_string(),
                        genre: app.ui.panels.ai_clipper.content_genre.clone(),
                        min_score: app.ui.panels.ai_clipper.min_virality_score,
                    });
                }
            });

            // Active Progress Box
            if is_busy || app.ui.panels.ai_clipper.status == "COMPLETED" || app.ui.panels.ai_clipper.status == "FAILED" {
                ui.add_space(6.0);
                egui::Frame::none()
                    .fill(Color32::from_rgb(26, 26, 36))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(60, 50, 90)))
                    .rounding(Rounding::same(6.0))
                    .inner_margin(egui::Margin::same(10.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(&app.ui.panels.ai_clipper.status)
                                    .strong()
                                    .small()
                                    .color(match app.ui.panels.ai_clipper.status.as_str() {
                                        "COMPLETED" => Color32::from_rgb(52, 211, 153),
                                        "FAILED" => Color32::from_rgb(248, 113, 113),
                                        _ => Color32::from_rgb(167, 139, 250),
                                    }),
                            );
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(
                                    egui::RichText::new(format!(
                                        "{}%",
                                        app.ui.panels.ai_clipper.progress
                                    ))
                                    .strong()
                                    .color(Color32::WHITE),
                                );
                            });
                        });

                        ui.add_space(4.0);
                        let prog_frac = (app.ui.panels.ai_clipper.progress as f32 / 100.0).clamp(0.0, 1.0);
                        ui.add(egui::ProgressBar::new(prog_frac).show_percentage());

                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new(if let Some(ref e) = app.ui.panels.ai_clipper.error_message {
                                format!("Error: {e}")
                            } else {
                                app.ui.panels.ai_clipper.current_step.clone()
                            })
                            .small()
                            .color(t.text_dim),
                        );
                    });
            }

            // Viral Clips Cards List
            if !app.ui.panels.ai_clipper.clips.is_empty() {
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(format!(
                        "🔥 Hasil Klip Viral ({} Klip)",
                        app.ui.panels.ai_clipper.clips.len()
                    ))
                    .font(FontId::proportional(15.0))
                    .strong()
                    .color(Color32::WHITE),
                );

                for clip in app.ui.panels.ai_clipper.clips.clone() {
                    egui::Frame::none()
                        .fill(Color32::from_rgb(30, 30, 42))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(50, 45, 70)))
                        .rounding(Rounding::same(8.0))
                        .inner_margin(egui::Margin::same(10.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(format!("⭐ {}", clip.virality_score))
                                        .strong()
                                        .small()
                                        .color(Color32::from_rgb(250, 204, 21)),
                                );
                                ui.label(
                                    egui::RichText::new(clip.layout_type.to_uppercase())
                                        .small()
                                        .color(Color32::from_rgb(167, 139, 250)),
                                );
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(
                                        egui::RichText::new(format!("{:.0}s", clip.duration_sec))
                                            .small()
                                            .color(t.text_dim),
                                    );
                                });
                            });

                            ui.add_space(2.0);
                            ui.label(
                                egui::RichText::new(&clip.title)
                                    .font(FontId::proportional(13.0))
                                    .strong()
                                    .color(Color32::WHITE),
                            );

                            if !clip.description.is_empty() {
                                ui.label(egui::RichText::new(&clip.description).small().color(t.text_dim));
                            }

                            ui.add_space(6.0);
                            ui.horizontal(|ui| {
                                // Add to Sequence button
                                let add_btn = egui::Button::new(
                                    egui::RichText::new("➕ Ke Timeline (9:16)").small().color(Color32::WHITE),
                                )
                                .fill(Color32::from_rgb(124, 58, 237));

                                if ui.add(add_btn).clicked() {
                                    // Ticket [DESKTOP-05]: Dispatch native engine commands to place 9:16 sequence
                                    let seq_name = format!("Shorts — {}", clip.title);
                                    let res = app.session.execute(
                                        "file.newSequence",
                                        json!({
                                            "name": seq_name,
                                            "width": 1080,
                                            "height": 1920,
                                            "fps": 30.0,
                                            "video": 2,
                                            "audio": 2
                                        }),
                                    );
                                    if let Ok(_) = res {
                                        let _ = app.session.execute(
                                            "captions.newTrack",
                                            json!({
                                                "format": "Subtitle",
                                                "name": "Auto Captions",
                                                "language": "id"
                                            }),
                                        );
                                        let _ = app.session.execute(
                                            "captions.setStyle",
                                            json!({
                                                "track": "C1",
                                                "size": 36.0,
                                                "anchor": "middle",
                                                "align": "center"
                                            }),
                                        );
                                        app.ui.status = format!("9:16 Sequence berhasil dibuat: {}", clip.title);
                                    } else if let Err(e) = res {
                                        app.ui.status = format!("Gagal membuat sequence: {e}");
                                    }
                                }

                                // Quick Export / Open link
                                let full_video_url = format!(
                                    "{}{}",
                                    app.ui.panels.ai_clipper.api_endpoint.trim_end_matches('/'),
                                    clip.video_url
                                );
                                if ui.button(egui::RichText::new("⬇ Download MP4").small()).clicked() {
                                    let _ = open::that(&full_video_url);
                                }
                            });
                        });
                    ui.add_space(4.0);
                }
            }
        });
}
