use fhre::{
    pipeline::{Sampler, TextureFormat},
    Assets, Events, Image, ResMut,
};

use crate::{
    resources::WingImageResources,
    theme::{
        AURORA_BACKGROUND_RGB565, DUSK_BACKGROUND_RGB565, ICON_AIRPLANE_A8, ICON_BLUETOOTH_A8,
        ICON_BRIGHTNESS_A8, ICON_DND_A8, ICON_FLASHLIGHT_A8, ICON_HOME_A8, ICON_PREVIEW_A8,
        ICON_ROTATE_A8, ICON_SETTINGS_A8, ICON_SIZE_LAUNCHER, ICON_SIZE_QUICK, ICON_SIZE_WING,
        ICON_TERMINAL_A8, ICON_THEME_A8, ICON_WIFI_A8, ICON_WING_A8, THEME_BACKGROUND_HEIGHT,
        THEME_BACKGROUND_WIDTH,
    },
};

pub fn setup_wing_image_resources(
    mut images: ResMut<Assets<Image>>,
    mut wing_images: ResMut<WingImageResources>,
    mut events: ResMut<Events>,
) {
    if wing_images.loaded {
        return;
    }

    wing_images.aurora_background = images.add_with_event(
        rgb565_image(
            AURORA_BACKGROUND_RGB565,
            THEME_BACKGROUND_WIDTH,
            THEME_BACKGROUND_HEIGHT,
        ),
        &mut events,
    );
    wing_images.dusk_background = images.add_with_event(
        rgb565_image(
            DUSK_BACKGROUND_RGB565,
            THEME_BACKGROUND_WIDTH,
            THEME_BACKGROUND_HEIGHT,
        ),
        &mut events,
    );

    wing_images.settings_icon = images.add_with_event(
        a8_image(ICON_SETTINGS_A8, ICON_SIZE_LAUNCHER, ICON_SIZE_LAUNCHER),
        &mut events,
    );
    wing_images.terminal_icon = images.add_with_event(
        a8_image(ICON_TERMINAL_A8, ICON_SIZE_LAUNCHER, ICON_SIZE_LAUNCHER),
        &mut events,
    );
    wing_images.wifi_icon = images.add_with_event(
        a8_image(ICON_WIFI_A8, ICON_SIZE_QUICK, ICON_SIZE_QUICK),
        &mut events,
    );
    wing_images.bluetooth_icon = images.add_with_event(
        a8_image(ICON_BLUETOOTH_A8, ICON_SIZE_QUICK, ICON_SIZE_QUICK),
        &mut events,
    );
    wing_images.airplane_icon = images.add_with_event(
        a8_image(ICON_AIRPLANE_A8, ICON_SIZE_QUICK, ICON_SIZE_QUICK),
        &mut events,
    );
    wing_images.flashlight_icon = images.add_with_event(
        a8_image(ICON_FLASHLIGHT_A8, ICON_SIZE_QUICK, ICON_SIZE_QUICK),
        &mut events,
    );
    wing_images.dnd_icon = images.add_with_event(
        a8_image(ICON_DND_A8, ICON_SIZE_QUICK, ICON_SIZE_QUICK),
        &mut events,
    );
    wing_images.rotate_icon = images.add_with_event(
        a8_image(ICON_ROTATE_A8, ICON_SIZE_QUICK, ICON_SIZE_QUICK),
        &mut events,
    );
    wing_images.brightness_icon = images.add_with_event(
        a8_image(ICON_BRIGHTNESS_A8, ICON_SIZE_QUICK, ICON_SIZE_QUICK),
        &mut events,
    );
    wing_images.theme_icon = images.add_with_event(
        a8_image(ICON_THEME_A8, ICON_SIZE_QUICK, ICON_SIZE_QUICK),
        &mut events,
    );
    wing_images.preview_icon = images.add_with_event(
        a8_image(ICON_PREVIEW_A8, ICON_SIZE_QUICK, ICON_SIZE_QUICK),
        &mut events,
    );
    wing_images.home_icon = images.add_with_event(
        a8_image(ICON_HOME_A8, ICON_SIZE_QUICK, ICON_SIZE_QUICK),
        &mut events,
    );
    wing_images.wing_icon = images.add_with_event(
        a8_image(ICON_WING_A8, ICON_SIZE_WING, ICON_SIZE_WING),
        &mut events,
    );
    wing_images.loaded = true;
}

fn rgb565_image(data: &'static [u8], width: u32, height: u32) -> Image {
    Image {
        data: data.to_vec(),
        width,
        height,
        format: TextureFormat::Rgb565,
        sampler: Sampler::NEAREST,
    }
}

fn a8_image(data: &'static [u8], width: u32, height: u32) -> Image {
    Image {
        data: data.to_vec(),
        width,
        height,
        format: TextureFormat::A8,
        sampler: Sampler::NEAREST,
    }
}
