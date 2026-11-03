//! Motion Lab's compact adapter: commands and panel bodies retain their existing owners.

use crate::dock::PanelKind;
use crate::mobile::{PREFERENCE_FILE, Pane};
use crate::state::Tool;
use crate::{EffectcraftApp, menus, panels};
use egui::{Rect, UiBuilder, vec2};
use serde_json::json;

impl EffectcraftApp {
    pub(crate) fn mobile_sync_context(&mut self) {
        let context = (self.session.script_ui.project_epoch, self.session.active_comp_id().map(|id| id.0));
        if self.mobile.context != Some(context) {
            self.mobile.context = Some(context);
            self.mobile.timeline = None;
            self.mobile.viewer = None;
            self.mobile_checkpoint = None;
        }
    }

    pub(crate) fn mobile_store_layout(&mut self) {
        if let Some(config) = self.session.config.as_deref()
            && let Err(error) = config.write(PREFERENCE_FILE, &self.mobile.preference())
        {
            self.ui.status = format!("Layout changed for this session; preference could not be saved: {error}");
        }
    }

    pub(crate) fn mobile_raw_input(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) {
        if !self.mobile.policy.compact {
            return;
        }
        let route = self.mobile.touch.filter(raw, self.mobile.bounds);
        if route.started && !self.mobile.navigate && self.dialog.is_none() {
            self.mobile_checkpoint = Some(effectcraft_engine::gesture::Checkpoint::capture(&self.session));
        }
        if route.cancel {
            if let Some(checkpoint) = self.mobile_checkpoint.take() {
                checkpoint.cancel(&mut self.session);
            }
            panels::viewer::cancel_touch(ctx);
            panels::timeline::cancel_touch(ctx);
        }
        if route.ended {
            self.mobile_checkpoint = None;
        }
    }

    pub(crate) fn mobile_command(&mut self, ctx: &egui::Context, command: &str) {
        let command = match command {
            "@new" => "comp.new",
            "@export" => "comp.saveFrameAs",
            "@play" => "playback.toggle",
            "@stop" => "playback.stop",
            command => command,
        };
        if let Err(error) = menus::invoke(self, ctx, command, json!({})) {
            self.ui.status = error;
        }
    }

    pub(crate) fn mobile_panel(&mut self, ui: &mut egui::Ui, pane: Pane, rect: Rect) {
        let mut child = ui.new_child(UiBuilder::new().max_rect(rect).id_salt(("mobile-panel", pane.name())));
        child.set_clip_rect(rect.intersect(ui.clip_rect()));
        child.spacing_mut().interact_size.y = 44.0;
        if self.mobile.touch.ownership.blocked {
            child.disable();
        }
        if pane == Pane::Tools {
            egui::ScrollArea::vertical().show(&mut child, |ui| {
                ui.heading("Motion tools");
                for tool in Tool::ALL {
                    let response = ui.add(egui::Button::new(tool.label()).selected(self.ui.tool == tool).min_size(vec2(220.0, 44.0)));
                    self.auto.add(&format!("mobile.tool.{}", tool.label()), response.rect, tool.label());
                    if response.clicked() {
                        self.ui.tool = tool;
                        self.mobile.navigate = false;
                        self.mobile.pane = Pane::Preview;
                    }
                }
            });
            return;
        }
        if pane == Pane::Preview && self.session.active_comp_id().is_none() {
            child.vertical_centered(|ui| {
                ui.heading("ACA Motion Lab");
                ui.label("Create a composition or open your saved project.");
                if ui.add(egui::Button::new("New composition").min_size(vec2(220.0, 44.0))).clicked() {
                    self.mobile_command(ui.ctx(), "@new");
                }
                if ui.add(egui::Button::new("Open project").min_size(vec2(220.0, 44.0))).clicked() {
                    self.mobile_command(ui.ctx(), "file.open");
                }
            });
            return;
        }
        let panel = match pane {
            Pane::Preview => PanelKind::Composition,
            Pane::Timeline => PanelKind::Timeline,
            Pane::Project => PanelKind::Project,
            Pane::Properties => PanelKind::Properties,
            Pane::Tools => return,
        };
        let old_focus = self.ui.focused;
        let old_tool = self.ui.tool;
        self.ui.focused = panel;
        if self.mobile.navigate && matches!(pane, Pane::Preview | Pane::Timeline) {
            self.ui.tool = Tool::Hand;
        }
        // Compact timeline navigation is its own runtime view. Automatic sizing cannot clamp
        // and then persist the user's desktop zoom/scroll/columns.
        if pane == Pane::Timeline {
            let compact = self.mobile.timeline.take().unwrap_or_else(|| self.ui.timeline.clone());
            let desktop = std::mem::replace(&mut self.ui.timeline, compact);
            self.panel_body(&mut child, panel, rect);
            self.mobile.timeline = Some(std::mem::replace(&mut self.ui.timeline, desktop));
        } else if pane == Pane::Preview {
            let compact = self.mobile.viewer.take().unwrap_or_else(|| self.ui.viewer.clone());
            let desktop = std::mem::replace(&mut self.ui.viewer, compact);
            // A locked or secondary desktop viewer must not make the compact editor passive.
            self.auto.add(&format!("panel.{}", PanelKind::Composition.id()), rect, "Composition");
            panels::viewer::show(self, &mut child, rect);
            self.mobile.viewer = Some(std::mem::replace(&mut self.ui.viewer, desktop));
        } else {
            self.panel_body(&mut child, panel, rect);
        }
        self.ui.focused = old_focus;
        self.ui.tool = old_tool;
    }
}
