use std::collections::HashMap;

use log::info;

use ibus_sys::bindings::{
    ibus_engine_hide_auxiliary_text, ibus_engine_hide_lookup_table, IBusEngine,
};
use ibus_sys::lookup_table::ibus_lookup_table_clear;

use crate::{_make_preedit_word, update_preedit_text_before_henkan, AkazaContext, InputMode};

pub type IbusAkazaCommand = fn(&mut AkazaContext, *mut IBusEngine);

macro_rules! command {
    ($i: ident) => {
        (stringify!($i), $i as IbusAkazaCommand)
    };
}
pub(crate) fn ibus_akaza_commands_map() -> HashMap<&'static str, IbusAkazaCommand> {
    HashMap::from([
        command!(commit_preedit),
        command!(erase_character_before_cursor),
        command!(set_input_mode_hiragana),
        command!(set_input_mode_alnum),
        command!(update_candidates),
    ])
}

fn commit_preedit(context: &mut AkazaContext, engine: *mut IBusEngine) {
    /*
    # 無変換状態では、ひらがなに変換してコミットします。
    yomi, word = self._make_preedit_word()
    self.commit_string(word)
     */
    unsafe {
        let (_, surface) = _make_preedit_word(context);
        context.commit_string(engine, surface.as_str());
    }
}

fn set_input_mode_hiragana(context: &mut AkazaContext, engine: *mut IBusEngine) {
    context.set_input_mode(InputMode::Hiragana, engine)
}

fn set_input_mode_alnum(context: &mut AkazaContext, engine: *mut IBusEngine) {
    context.set_input_mode(InputMode::Alnum, engine)
}

fn update_candidates(context: &mut AkazaContext, engine: *mut IBusEngine) {
    context.update_candidates(engine)
}

fn erase_character_before_cursor(context: &mut AkazaContext, engine: *mut IBusEngine) {
    info!("erase_character_before_cursor: {}", context.preedit);

    // TODO move this to context.rs
    unsafe {
        if context.in_henkan_mode() {
            // 変換中の場合、無変換モードにもどす。
            ibus_lookup_table_clear(context.lookup_table);
            ibus_engine_hide_auxiliary_text(engine);
            ibus_engine_hide_lookup_table(engine);
        } else {
            // サイゴの一文字をけずるが、子音が先行しているばあいは、子音もついでにとる。
            context.preedit = context.romkan.remove_last_char(&context.preedit)
        }
        // 変換していないときのレンダリングをする。
        update_preedit_text_before_henkan(context, engine);
    }
    /*
       self.logger.info(f"erase_character_before_cursor: {self.preedit_string}")
       if self.in_henkan_mode():
           # 変換中の場合、無変換モードにもどす。
           self.lookup_table.clear()
           self.hide_auxiliary_text()
           self.hide_lookup_table()
       else:
           # サイゴの一文字をけずるが、子音が先行しているばあいは、子音もついでにとる。
           self.preedit_string = self.romkan.remove_last_char(self.preedit_string)
       # 変換していないときのレンダリングをする。
       self.update_preedit_text_before_henkan()
    */
}
