//! UI chrome localization.
//!
//! Only the built-in chrome strings (buttons, dialog titles, placeholders) are
//! translated here. Schema-defined `label` / `hint` / `option` text is supplied
//! verbatim by the schema author and is never routed through this module.
//!
//! The active language is fixed once at startup (it does not change while the
//! process runs), so the resolved [`Strings`] table is stored in a process-wide
//! [`OnceLock`] and accessed through [`t`] without threading it through every
//! render function.
//!
//! # Schema overrides
//!
//! Schema authors may supply a `[ui_strings]` table to override any built-in
//! string. Fields not present in that table fall back to the built-in translation
//! for the active language (or English when the language has no built-in entry).
//! Overridden strings are leaked at startup so they share the same `'static`
//! lifetime as the built-in constants; this is intentional since they live for
//! the entire process lifetime.

use std::sync::OnceLock;

use crate::schema::{LangMode, UiStrings};

// ---------------------------------------------------------------------------
// Lang

/// Supported UI languages. Languages listed here have built-in translations;
/// other BCP-47 codes accepted via `lang = "<code>"` fall back to English
/// unless the schema provides overrides via `[ui_strings]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Ar,
    Zh,
    De,
    En,
    Fr,
    Hi,
    It,
    Ja,
    Ko,
    Nl,
    Pt,
    Ru,
    Es,
    Sv,
    Tr,
    Vi,
}

impl Lang {
    /// Resolves the effective UI language.
    ///
    /// * [`LangMode::Fixed`] — explicit code; parsed via [`parse_lang_tag`],
    ///   falls back to English for unrecognized codes.
    /// * [`LangMode::Os`] — first matches the parent application's language
    ///   (`parent_lang`, read from the config file via the schema's `lang_key`),
    ///   so the settings window follows the parent; if that is absent or
    ///   unrecognized, falls back to native OS detection, then to English.
    pub fn resolve(mode: &LangMode, parent_lang: Option<&str>) -> Lang {
        match mode {
            LangMode::Fixed(code) => parse_lang_tag(code).unwrap_or(Lang::En),
            LangMode::Os => parent_lang
                .and_then(parse_lang_tag)
                .or_else(detect_os_lang)
                .unwrap_or(Lang::En),
        }
    }

    /// The BCP-47 base language code, used to select localized schema strings.
    pub fn code(self) -> &'static str {
        match self {
            Lang::Ar => "ar",
            Lang::Zh => "zh",
            Lang::De => "de",
            Lang::En => "en",
            Lang::Fr => "fr",
            Lang::Hi => "hi",
            Lang::It => "it",
            Lang::Ja => "ja",
            Lang::Ko => "ko",
            Lang::Nl => "nl",
            Lang::Pt => "pt",
            Lang::Ru => "ru",
            Lang::Es => "es",
            Lang::Sv => "sv",
            Lang::Tr => "tr",
            Lang::Vi => "vi",
        }
    }
}

/// Maps a BCP-47 / POSIX language tag (e.g. `"ja"`, `"ja-JP"`, `"en_US"`) to a
/// [`Lang`]. Returns `None` for empty, placeholder (`"c"` / `"posix"`), or
/// unrecognized tags so the caller can fall through to the next detection source.
fn parse_lang_tag(tag: &str) -> Option<Lang> {
    let tag = tag.trim().to_ascii_lowercase();
    if tag.is_empty() || tag == "c" || tag == "posix" {
        return None;
    }
    // Match on the primary language subtag only (everything before '-' or '_').
    let primary = tag.split(|c| c == '-' || c == '_').next().unwrap_or(&tag);
    match primary {
        "ar" => Some(Lang::Ar),
        "zh" => Some(Lang::Zh),
        "de" => Some(Lang::De),
        "en" => Some(Lang::En),
        "fr" => Some(Lang::Fr),
        "hi" => Some(Lang::Hi),
        "it" => Some(Lang::It),
        "ja" => Some(Lang::Ja),
        "ko" => Some(Lang::Ko),
        "nl" => Some(Lang::Nl),
        "pt" => Some(Lang::Pt),
        "ru" => Some(Lang::Ru),
        "es" => Some(Lang::Es),
        "sv" => Some(Lang::Sv),
        "tr" => Some(Lang::Tr),
        "vi" => Some(Lang::Vi),
        _ => None,
    }
}

/// Detects the OS UI language using the most accurate native source per platform.
///
/// * macOS — `NSLocale.preferredLanguages`, which already reflects the per-app
///   override (System Settings → Language & Region → Applications) and then the
///   system-wide Preferred Languages list.
/// * Windows — `GetUserPreferredUILanguages` (the display language).
/// * Other (Linux/BSD) — the `LC_ALL` / `LC_MESSAGES` / `LANG` environment
///   variables in POSIX precedence order.
#[cfg(target_os = "macos")]
fn detect_os_lang() -> Option<Lang> {
    use objc2_foundation::NSLocale;
    for tag in NSLocale::preferredLanguages() {
        if let Some(lang) = parse_lang_tag(&tag.to_string()) {
            return Some(lang);
        }
    }
    None
}

#[cfg(target_os = "windows")]
fn detect_os_lang() -> Option<Lang> {
    use windows_sys::Win32::Foundation::FALSE;
    use windows_sys::Win32::Globalization::GetUserPreferredUILanguages;
    const MUI_LANGUAGE_NAME: u32 = 0x08;
    unsafe {
        let mut num_langs: u32 = 0;
        let mut buf_size: u32 = 0;
        GetUserPreferredUILanguages(
            MUI_LANGUAGE_NAME,
            &mut num_langs,
            std::ptr::null_mut(),
            &mut buf_size,
        );
        if buf_size == 0 {
            return None;
        }
        let mut buf: Vec<u16> = vec![0u16; buf_size as usize];
        let ok = GetUserPreferredUILanguages(
            MUI_LANGUAGE_NAME,
            &mut num_langs,
            buf.as_mut_ptr(),
            &mut buf_size,
        );
        if ok == FALSE {
            return None;
        }
        for segment in buf.split(|&c| c == 0) {
            if segment.is_empty() {
                continue;
            }
            let tag = String::from_utf16_lossy(segment);
            if let Some(lang) = parse_lang_tag(&tag) {
                return Some(lang);
            }
        }
    }
    None
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn detect_os_lang() -> Option<Lang> {
    for var in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(val) = std::env::var(var) {
            if let Some(lang) = parse_lang_tag(&val) {
                return Some(lang);
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Strings

/// All built-in UI strings for one language. `Copy` because every field is a
/// `&'static str`, so passing it around is free.
#[derive(Debug, Clone, Copy)]
pub struct Strings {
    pub ok: &'static str,
    pub apply: &'static str,
    pub no_sections: &'static str,
    pub add_section: &'static str,
    pub section_name_label: &'static str,
    pub add: &'static str,
    pub cancel: &'static str,
    pub enter_name: &'static str,
    pub delete: &'static str,
    /// Template for the delete confirmation. Use [`Strings::delete_confirm`].
    delete_confirm_tmpl: &'static str,
    pub browse: &'static str,
    /// "All files" entry added to the dialog filter dropdown. The filter is
    /// compiled out on macOS, so this string is unused there.
    #[cfg_attr(target_os = "macos", allow(dead_code))]
    pub all_files: &'static str,
    pub click_to_input: &'static str,
    pub press_key: &'static str,
    pub clear: &'static str,
    pub show: &'static str,
    pub hide: &'static str,
    /// Body text shown in the external-change conflict dialog.
    pub file_changed: &'static str,
    /// "Reload" button label in the conflict dialog.
    pub reload: &'static str,
    /// "Keep Editing" button label in the conflict dialog.
    pub keep_editing: &'static str,
    /// Window title bar text.
    pub window_title: &'static str,
}

impl Strings {
    /// Builds the delete confirmation message for `name`.
    pub fn delete_confirm(&self, name: &str) -> String {
        self.delete_confirm_tmpl.replace("{}", name)
    }
}

// ---------------------------------------------------------------------------
// Built-in string tables

const AR: Strings = Strings {
    ok: "موافق",
    apply: "تطبيق",
    no_sections: "(لا توجد أقسام)",
    add_section: "إضافة قسم",
    section_name_label: "اسم القسم:",
    add: "إضافة",
    cancel: "إلغاء",
    enter_name: "الرجاء إدخال اسم",
    delete: "حذف",
    delete_confirm_tmpl: "هل تريد حذف \"{}\"؟",
    browse: "استعراض…",
    all_files: "كل الملفات",
    click_to_input: "انقر للإدخال…",
    press_key: "اضغط مفتاحًا…",
    clear: "مسح",
    show: "إظهار",
    hide: "إخفاء",
    file_changed: "تم تعديل ملف الإعدادات خارجيًا. سيؤدي إعادة التحميل إلى تجاهل تعديلاتك الحالية. إذا واصلت التعديل وحفظت، ستُستبدل التغييرات الخارجية.",
    reload: "إعادة تحميل",
    keep_editing: "مواصلة التعديل",
    window_title: "الإعدادات",
};

const ZH: Strings = Strings {
    ok: "确定",
    apply: "应用",
    no_sections: "（无分区）",
    add_section: "添加分区",
    section_name_label: "分区名称：",
    add: "添加",
    cancel: "取消",
    enter_name: "请输入名称",
    delete: "删除",
    delete_confirm_tmpl: "删除\"{}\"？",
    browse: "浏览…",
    all_files: "所有文件",
    click_to_input: "点击输入…",
    press_key: "请按键…",
    clear: "清除",
    show: "显示",
    hide: "隐藏",
    file_changed: "配置文件已被外部修改。重新加载将丢弃您当前的编辑内容。继续编辑并保存将覆盖外部的更改。",
    reload: "重新加载",
    keep_editing: "继续编辑",
    window_title: "设置",
};

const DE: Strings = Strings {
    ok: "OK",
    apply: "Übernehmen",
    no_sections: "(keine Abschnitte)",
    add_section: "Abschnitt hinzufügen",
    section_name_label: "Abschnittsname:",
    add: "Hinzufügen",
    cancel: "Abbrechen",
    enter_name: "Bitte einen Namen eingeben",
    delete: "Löschen",
    delete_confirm_tmpl: "\"{}\" löschen?",
    browse: "Durchsuchen…",
    all_files: "Alle Dateien",
    click_to_input: "Klicken zum Eingeben…",
    press_key: "Taste drücken…",
    clear: "Leeren",
    show: "Anzeigen",
    hide: "Ausblenden",
    file_changed: "Die Konfigurationsdatei wurde extern geändert. Beim Neuladen gehen Ihre aktuellen Änderungen verloren. Wenn Sie weiterbearbeiten und speichern, werden die externen Änderungen überschrieben.",
    reload: "Neu laden",
    keep_editing: "Weiterbearbeiten",
    window_title: "Einstellungen",
};

const EN: Strings = Strings {
    ok: "OK",
    apply: "Apply",
    no_sections: "(no sections)",
    add_section: "Add section",
    section_name_label: "Section name:",
    add: "Add",
    cancel: "Cancel",
    enter_name: "Please enter a name",
    delete: "Delete",
    delete_confirm_tmpl: "Delete \"{}\"?",
    browse: "Browse…",
    all_files: "All files",
    click_to_input: "Click to enter…",
    press_key: "Press a key…",
    clear: "Clear",
    show: "Show",
    hide: "Hide",
    file_changed: "The config file was modified externally. Reloading will discard your current edits. Continuing to edit and then saving will overwrite the external changes.",
    reload: "Reload",
    keep_editing: "Keep Editing",
    window_title: "Settings",
};

const FR: Strings = Strings {
    ok: "OK",
    apply: "Appliquer",
    no_sections: "(aucune section)",
    add_section: "Ajouter une section",
    section_name_label: "Nom de la section :",
    add: "Ajouter",
    cancel: "Annuler",
    enter_name: "Veuillez entrer un nom",
    delete: "Supprimer",
    delete_confirm_tmpl: "Supprimer \"{}\" ?",
    browse: "Parcourir…",
    all_files: "Tous les fichiers",
    click_to_input: "Cliquer pour saisir…",
    press_key: "Appuyez sur une touche…",
    clear: "Effacer",
    show: "Afficher",
    hide: "Masquer",
    file_changed: "Le fichier de configuration a été modifié en dehors de l'application. Recharger supprimera vos modifications en cours. Continuer à modifier et sauvegarder écrasera les modifications externes.",
    reload: "Recharger",
    keep_editing: "Continuer à modifier",
    window_title: "Préférences",
};

const HI: Strings = Strings {
    ok: "ठीक है",
    apply: "लागू करें",
    no_sections: "(कोई अनुभाग नहीं)",
    add_section: "अनुभाग जोड़ें",
    section_name_label: "अनुभाग का नाम:",
    add: "जोड़ें",
    cancel: "रद्द करें",
    enter_name: "कृपया एक नाम दर्ज करें",
    delete: "हटाएं",
    delete_confirm_tmpl: "\"{}\" हटाएं?",
    browse: "ब्राउज़ करें…",
    all_files: "सभी फ़ाइलें",
    click_to_input: "इनपुट के लिए क्लिक करें…",
    press_key: "एक कुंजी दबाएं…",
    clear: "साफ़ करें",
    show: "दिखाएं",
    hide: "छुपाएं",
    file_changed: "कॉन्फ़िग फ़ाइल को बाहरी रूप से संशोधित किया गया। पुनः लोड करने पर आपके वर्तमान संपादन हट जाएंगे। संपादन जारी रखकर सहेजने पर बाहरी परिवर्तन अधिलेखित हो जाएंगे।",
    reload: "पुनः लोड करें",
    keep_editing: "संपादन जारी रखें",
    window_title: "सेटिंग्स",
};

const IT: Strings = Strings {
    ok: "OK",
    apply: "Applica",
    no_sections: "(nessuna sezione)",
    add_section: "Aggiungi sezione",
    section_name_label: "Nome sezione:",
    add: "Aggiungi",
    cancel: "Annulla",
    enter_name: "Inserire un nome",
    delete: "Elimina",
    delete_confirm_tmpl: "Eliminare \"{}\"?",
    browse: "Sfoglia…",
    all_files: "Tutti i file",
    click_to_input: "Clicca per inserire…",
    press_key: "Premi un tasto…",
    clear: "Cancella",
    show: "Mostra",
    hide: "Nascondi",
    file_changed: "Il file di configurazione è stato modificato esternamente. Ricaricare eliminerà le modifiche correnti. Continuare a modificare e salvare sovrascriverà le modifiche esterne.",
    reload: "Ricarica",
    keep_editing: "Continua a modificare",
    window_title: "Impostazioni",
};

const JA: Strings = Strings {
    ok: "OK",
    apply: "適用",
    no_sections: "(セクションなし)",
    add_section: "セクションを追加",
    section_name_label: "セクション名:",
    add: "追加",
    cancel: "キャンセル",
    enter_name: "名前を入力してください",
    delete: "削除",
    delete_confirm_tmpl: "「{}」を削除しますか？",
    browse: "参照…",
    all_files: "すべてのファイル",
    click_to_input: "クリックして入力…",
    press_key: "キーを押してください…",
    clear: "クリア",
    show: "表示",
    hide: "隠す",
    file_changed: "設定ファイルが外部で変更されました。再読み込みすると現在の編集内容は失われます。編集を続けて保存すると，外部での変更は上書きされます。",
    reload: "再読み込み",
    keep_editing: "編集を続ける",
    window_title: "設定",
};

const KO: Strings = Strings {
    ok: "확인",
    apply: "적용",
    no_sections: "(섹션 없음)",
    add_section: "섹션 추가",
    section_name_label: "섹션 이름:",
    add: "추가",
    cancel: "취소",
    enter_name: "이름을 입력하세요",
    delete: "삭제",
    delete_confirm_tmpl: "\"{}\"을(를) 삭제하시겠습니까?",
    browse: "찾아보기…",
    all_files: "모든 파일",
    click_to_input: "클릭하여 입력…",
    press_key: "키를 누르세요…",
    clear: "지우기",
    show: "표시",
    hide: "숨기기",
    file_changed: "설정 파일이 외부에서 수정되었습니다. 다시 불러오면 현재 편집 내용이 삭제됩니다. 계속 편집하여 저장하면 외부 변경 사항이 덮어씌워집니다.",
    reload: "다시 불러오기",
    keep_editing: "계속 편집",
    window_title: "설정",
};

const NL: Strings = Strings {
    ok: "OK",
    apply: "Toepassen",
    no_sections: "(geen secties)",
    add_section: "Sectie toevoegen",
    section_name_label: "Sectienaam:",
    add: "Toevoegen",
    cancel: "Annuleren",
    enter_name: "Voer een naam in",
    delete: "Verwijderen",
    delete_confirm_tmpl: "\"{}\" verwijderen?",
    browse: "Bladeren…",
    all_files: "Alle bestanden",
    click_to_input: "Klik om in te voeren…",
    press_key: "Druk op een toets…",
    clear: "Wissen",
    show: "Tonen",
    hide: "Verbergen",
    file_changed: "Het configuratiebestand is extern gewijzigd. Herladen verwijdert uw huidige wijzigingen. Als u doorgaat met bewerken en opslaat, worden de externe wijzigingen overschreven.",
    reload: "Herladen",
    keep_editing: "Doorgaan met bewerken",
    window_title: "Instellingen",
};

const PT: Strings = Strings {
    ok: "OK",
    apply: "Aplicar",
    no_sections: "(sem seções)",
    add_section: "Adicionar seção",
    section_name_label: "Nome da seção:",
    add: "Adicionar",
    cancel: "Cancelar",
    enter_name: "Por favor, insira um nome",
    delete: "Excluir",
    delete_confirm_tmpl: "Excluir \"{}\"?",
    browse: "Procurar…",
    all_files: "Todos os arquivos",
    click_to_input: "Clique para inserir…",
    press_key: "Pressione uma tecla…",
    clear: "Limpar",
    show: "Mostrar",
    hide: "Ocultar",
    file_changed: "O arquivo de configuração foi modificado externamente. Recarregar descartará suas edições atuais. Continuar editando e salvar sobrescreverá as alterações externas.",
    reload: "Recarregar",
    keep_editing: "Continuar editando",
    window_title: "Configurações",
};

const RU: Strings = Strings {
    ok: "ОК",
    apply: "Применить",
    no_sections: "(нет разделов)",
    add_section: "Добавить раздел",
    section_name_label: "Название раздела:",
    add: "Добавить",
    cancel: "Отмена",
    enter_name: "Введите имя",
    delete: "Удалить",
    delete_confirm_tmpl: "Удалить \"{}\"?",
    browse: "Обзор…",
    all_files: "Все файлы",
    click_to_input: "Щёлкните для ввода…",
    press_key: "Нажмите клавишу…",
    clear: "Очистить",
    show: "Показать",
    hide: "Скрыть",
    file_changed: "Файл конфигурации был изменён внешней программой. Перезагрузка удалит текущие изменения. Продолжение редактирования с последующим сохранением перезапишет внешние изменения.",
    reload: "Перезагрузить",
    keep_editing: "Продолжить редактирование",
    window_title: "Настройки",
};

const ES: Strings = Strings {
    ok: "Aceptar",
    apply: "Aplicar",
    no_sections: "(sin secciones)",
    add_section: "Agregar sección",
    section_name_label: "Nombre de sección:",
    add: "Agregar",
    cancel: "Cancelar",
    enter_name: "Por favor, ingrese un nombre",
    delete: "Eliminar",
    delete_confirm_tmpl: "¿Eliminar \"{}\"?",
    browse: "Examinar…",
    all_files: "Todos los archivos",
    click_to_input: "Haga clic para ingresar…",
    press_key: "Presione una tecla…",
    clear: "Borrar",
    show: "Mostrar",
    hide: "Ocultar",
    file_changed: "El archivo de configuración fue modificado externamente. Recargar descartará sus ediciones actuales. Si continúa editando y guarda, los cambios externos serán sobrescritos.",
    reload: "Recargar",
    keep_editing: "Continuar editando",
    window_title: "Ajustes",
};

const SV: Strings = Strings {
    ok: "OK",
    apply: "Verkställ",
    no_sections: "(inga avsnitt)",
    add_section: "Lägg till avsnitt",
    section_name_label: "Avsnittets namn:",
    add: "Lägg till",
    cancel: "Avbryt",
    enter_name: "Ange ett namn",
    delete: "Ta bort",
    delete_confirm_tmpl: "Ta bort \"{}\"?",
    browse: "Bläddra…",
    all_files: "Alla filer",
    click_to_input: "Klicka för att ange…",
    press_key: "Tryck på en tangent…",
    clear: "Rensa",
    show: "Visa",
    hide: "Dölj",
    file_changed: "Konfigurationsfilen har ändrats externt. Att ladda om tar bort dina aktuella redigeringar. Om du fortsätter redigera och sparar skrivs de externa ändringarna över.",
    reload: "Ladda om",
    keep_editing: "Fortsätt redigera",
    window_title: "Inställningar",
};

const TR: Strings = Strings {
    ok: "Tamam",
    apply: "Uygula",
    no_sections: "(bölüm yok)",
    add_section: "Bölüm ekle",
    section_name_label: "Bölüm adı:",
    add: "Ekle",
    cancel: "İptal",
    enter_name: "Lütfen bir ad girin",
    delete: "Sil",
    delete_confirm_tmpl: "\"{}\" silinsin mi?",
    browse: "Gözat…",
    all_files: "Tüm dosyalar",
    click_to_input: "Girmek için tıklayın…",
    press_key: "Bir tuşa basın…",
    clear: "Temizle",
    show: "Göster",
    hide: "Gizle",
    file_changed: "Yapılandırma dosyası harici olarak değiştirildi. Yeniden yüklemek mevcut düzenlemelerinizi siler. Düzenlemeye devam edip kaydederseniz, harici değişiklikler üzerine yazılır.",
    reload: "Yeniden yükle",
    keep_editing: "Düzenlemeye devam et",
    window_title: "Ayarlar",
};

const VI: Strings = Strings {
    ok: "OK",
    apply: "Áp dụng",
    no_sections: "(không có mục)",
    add_section: "Thêm mục",
    section_name_label: "Tên mục:",
    add: "Thêm",
    cancel: "Hủy",
    enter_name: "Vui lòng nhập tên",
    delete: "Xóa",
    delete_confirm_tmpl: "Xóa \"{}\"?",
    browse: "Duyệt…",
    all_files: "Tất cả tệp",
    click_to_input: "Nhấp để nhập…",
    press_key: "Nhấn phím…",
    clear: "Xóa",
    show: "Hiện",
    hide: "Ẩn",
    file_changed: "Tệp cấu hình đã bị sửa đổi bên ngoài. Tải lại sẽ xóa các chỉnh sửa hiện tại của bạn. Tiếp tục chỉnh sửa và lưu sẽ ghi đè các thay đổi bên ngoài.",
    reload: "Tải lại",
    keep_editing: "Tiếp tục chỉnh sửa",
    window_title: "Cài đặt",
};

fn builtin(lang: Lang) -> &'static Strings {
    match lang {
        Lang::Ar => &AR,
        Lang::Zh => &ZH,
        Lang::De => &DE,
        Lang::En => &EN,
        Lang::Fr => &FR,
        Lang::Hi => &HI,
        Lang::It => &IT,
        Lang::Ja => &JA,
        Lang::Ko => &KO,
        Lang::Nl => &NL,
        Lang::Pt => &PT,
        Lang::Ru => &RU,
        Lang::Es => &ES,
        Lang::Sv => &SV,
        Lang::Tr => &TR,
        Lang::Vi => &VI,
    }
}

/// Leaks a string onto the heap, giving it a `'static` lifetime.
/// Used only at startup for schema-provided override strings, which must live
/// for the entire process lifetime anyway.
fn leak_str(s: &str) -> &'static str {
    Box::leak(s.to_owned().into_boxed_str())
}

/// Applies schema overrides on top of a built-in string table.
/// Returns the base table unchanged when no overrides are present;
/// otherwise leaks a new `Strings` onto the heap.
fn merge(base: &'static Strings, ov: &UiStrings) -> &'static Strings {
    macro_rules! field {
        ($f:ident) => {
            ov.$f.as_deref().map_or(base.$f, leak_str)
        };
    }

    if ov.ok.is_none()
        && ov.apply.is_none()
        && ov.no_sections.is_none()
        && ov.add_section.is_none()
        && ov.section_name_label.is_none()
        && ov.add.is_none()
        && ov.cancel.is_none()
        && ov.enter_name.is_none()
        && ov.delete.is_none()
        && ov.delete_confirm.is_none()
        && ov.browse.is_none()
        && ov.all_files.is_none()
        && ov.click_to_input.is_none()
        && ov.press_key.is_none()
        && ov.clear.is_none()
        && ov.show.is_none()
        && ov.hide.is_none()
        && ov.file_changed.is_none()
        && ov.reload.is_none()
        && ov.keep_editing.is_none()
        && ov.window_title.is_none()
    {
        return base;
    }

    Box::leak(Box::new(Strings {
        ok: field!(ok),
        apply: field!(apply),
        no_sections: field!(no_sections),
        add_section: field!(add_section),
        section_name_label: field!(section_name_label),
        add: field!(add),
        cancel: field!(cancel),
        enter_name: field!(enter_name),
        delete: field!(delete),
        delete_confirm_tmpl: ov.delete_confirm.as_deref().map_or(base.delete_confirm_tmpl, leak_str),
        browse: field!(browse),
        all_files: field!(all_files),
        click_to_input: field!(click_to_input),
        press_key: field!(press_key),
        clear: field!(clear),
        show: field!(show),
        hide: field!(hide),
        file_changed: field!(file_changed),
        reload: field!(reload),
        keep_editing: field!(keep_editing),
        window_title: field!(window_title),
    }))
}

// ---------------------------------------------------------------------------
// Process-wide state

static STRINGS: OnceLock<&'static Strings> = OnceLock::new();
static LANG: OnceLock<Lang> = OnceLock::new();

/// Initializes the process-wide string table. Call once at startup.
/// Subsequent calls are ignored (the first language wins).
pub fn init(lang: Lang, ui_strings: &UiStrings) {
    let _ = LANG.set(lang);
    settings_schema::set_active_lang_code(lang.code());
    let _ = STRINGS.set(merge(builtin(lang), ui_strings));
}

/// Returns the active string table. Defaults to English if [`init`] was not called.
pub fn t() -> &'static Strings {
    STRINGS.get().copied().unwrap_or(&EN)
}

/// Returns the active language code (e.g. `"ja"`, `"en"`, `"de"`). Defaults to
/// `"en"` if [`init`] was not called. Used by the schema to select localized strings.
pub fn active_lang_code() -> &'static str {
    LANG.get().copied().unwrap_or(Lang::En).code()
}
