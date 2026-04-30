use fhre::SvgId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActionId(pub u16);

pub const ACTION_DRAW: ActionId = ActionId(100);
pub const ACTION_DIRTY: ActionId = ActionId(101);
pub const ACTION_INPUT: ActionId = ActionId(102);
pub const ACTION_FONT: ActionId = ActionId(103);
pub const ACTION_SVG: ActionId = ActionId(104);
pub const ACTION_ECS: ActionId = ActionId(105);
pub const ACTION_SETTINGS: ActionId = ActionId(106);
pub const ACTION_ALL_APPS: ActionId = ActionId(107);
pub const ACTION_SETTINGS_SYSTEM: ActionId = ActionId(108);
pub const ACTION_SETTINGS_PERSONALIZATION: ActionId = ActionId(109);
pub const ACTION_SETTINGS_NETWORK: ActionId = ActionId(110);
pub const ACTION_SETTINGS_ABOUT: ActionId = ActionId(111);
pub const ACTION_HOME: ActionId = ActionId(112);
pub const ACTION_CORTANA: ActionId = ActionId(113);
pub const ACTION_LOCK_SCREEN: ActionId = ActionId(114);
pub const ACTION_SETTINGS_ACCOUNT: ActionId = ActionId(115);
pub const ACTION_SETTINGS_APPS: ActionId = ActionId(116);
pub const ACTION_SETTINGS_DEVICES: ActionId = ActionId(117);
pub const ACTION_SETTINGS_PRIVACY: ActionId = ActionId(118);
pub const ACTION_SETTINGS_TIME: ActionId = ActionId(119);
pub const ACTION_APP_PHONE: ActionId = ActionId(130);
pub const ACTION_APP_PEOPLE: ActionId = ActionId(131);
pub const ACTION_APP_MESSAGING: ActionId = ActionId(132);
pub const ACTION_APP_GROOVE: ActionId = ActionId(133);
pub const ACTION_APP_NEWS: ActionId = ActionId(134);
pub const ACTION_APP_PHOTOS: ActionId = ActionId(135);
pub const ACTION_APP_STARS: ActionId = ActionId(136);
pub const ACTION_APP_THERMAL: ActionId = ActionId(137);

pub const ICON_DRAW: SvgId = SvgId(1);
pub const ICON_DIRTY: SvgId = SvgId(2);
pub const ICON_INPUT: SvgId = SvgId(3);
pub const ICON_FONT: SvgId = SvgId(4);
pub const ICON_SVG: SvgId = SvgId(5);
pub const ICON_ECS: SvgId = SvgId(6);
pub const ICON_SETTINGS: SvgId = SvgId(7);

pub const fn action_label(action: ActionId) -> &'static str {
    match action {
        ACTION_DRAW => "ACTION: DRAW",
        ACTION_DIRTY => "ACTION: DIRTY",
        ACTION_INPUT => "ACTION: INPUT",
        ACTION_FONT => "ACTION: FONT",
        ACTION_SVG => "ACTION: SVG",
        ACTION_ECS => "ACTION: ECS",
        ACTION_SETTINGS => "ACTION: SETTINGS",
        ACTION_ALL_APPS => "ACTION: ALL APPS",
        ACTION_SETTINGS_SYSTEM => "ACTION: SYSTEM",
        ACTION_SETTINGS_PERSONALIZATION => "ACTION: PERSONALIZE",
        ACTION_SETTINGS_NETWORK => "ACTION: NETWORK",
        ACTION_SETTINGS_ABOUT => "ACTION: ABOUT",
        ACTION_HOME => "ACTION: HOME",
        ACTION_CORTANA => "ACTION: CORTANA",
        ACTION_LOCK_SCREEN => "ACTION: LOCK",
        ACTION_SETTINGS_ACCOUNT => "ACTION: ACCOUNT",
        ACTION_SETTINGS_APPS => "ACTION: APPS",
        ACTION_SETTINGS_DEVICES => "ACTION: DEVICES",
        ACTION_SETTINGS_PRIVACY => "ACTION: PRIVACY",
        ACTION_SETTINGS_TIME => "ACTION: TIME",
        ACTION_APP_PHONE => "ACTION: PHONE",
        ACTION_APP_PEOPLE => "ACTION: PEOPLE",
        ACTION_APP_MESSAGING => "ACTION: MESSAGING",
        ACTION_APP_GROOVE => "ACTION: GROOVE",
        ACTION_APP_NEWS => "ACTION: NEWS",
        ACTION_APP_PHOTOS => "ACTION: PHOTOS",
        ACTION_APP_STARS => "ACTION: STARS",
        ACTION_APP_THERMAL => "ACTION: THERMAL",
        _ => "ACTION: UNKNOWN",
    }
}
