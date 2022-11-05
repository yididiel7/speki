use tui::style::Color;

use crate::{
    app::{PopUpState, PopupValue, Tab, TabData, Widget},
    utils::{ankitemplate::ImportProgress, misc::split_updown_by_percent},
    widgets::progress_bar::ProgressBar,
};
use std::sync::mpsc::Receiver;

pub struct Progress {
    bar: ProgressBar,
    title: String,
    rx: Receiver<ImportProgress>,
    tabdata: TabData,
    popupvalue: PopupValue,
}

impl Progress {
    pub fn new(rx: Receiver<ImportProgress>, title: String) -> Self {
        Self {
            bar: ProgressBar::new(0),
            title,
            rx,
            tabdata: TabData::default(),
            popupvalue: PopupValue::None,
        }
    }
}

impl Tab for Progress {
    fn get_tabdata(&mut self) -> &mut TabData {
        &mut self.tabdata
    }
    fn keyhandler(
        &mut self,
        _appdata: &crate::app::AppData,
        _key: crate::MyKey,
        _cursor: &(u16, u16),
    ) {
    }

    fn render(
        &mut self,
        f: &mut tui::Frame<crate::MyType>,
        appdata: &crate::app::AppData,
        _cursor: &(u16, u16),
    ) {
        if let Ok(prog) = self.rx.recv() {
            let current = prog.curr_index as u32;
            self.bar.current = current;
            self.bar.max = prog.max as u32;
            //let rgb = get_rgb(current);
            self.bar.color = Color::Gray; //Color::Rgb(rgb.0, rgb.1, rgb.2);
            self.bar.render(f, appdata, &(0, 0));

            if prog.curr_index == prog.max - 1 {
                self.popupvalue = PopupValue::Ok;
                self.tabdata.state = PopUpState::Exit;
            }
        } else {
            self.popupvalue = PopupValue::Ok;
            self.tabdata.state = PopUpState::Exit;
        }
    }

    fn set_selection(&mut self, area: tui::layout::Rect) {
        let chunks = split_updown_by_percent([10, 20, 70], area);
        self.tabdata.view.areas.push(chunks[1]);
        self.bar.set_area(chunks[1]);
    }

    fn get_title(&self) -> String {
        self.title.clone()
    }
}
