use fhre::ResMut;

use crate::WingRuntime;

pub fn wing_update_system(mut runtime: ResMut<WingRuntime>) {
    runtime.wing.update(1.0 / 60.0);
}
