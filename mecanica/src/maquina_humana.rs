//! The human machine: named joints and Winter's segment fractions.
//!
//! This is the **only** module that knows what a hip is. Below it,
//! `garust` has a kinematic tree, forces and moments and no idea that a
//! gym exists; above it, a comparison knows which joint an exercise
//! trains but not how long a femur is. Putting anatomy anywhere else is
//! what makes a kernel un-reusable.
//!
//! Two things live here and nothing else:
//!
//! 1. **Anthropometry** — segment lengths as fractions of stature and
//!    segment masses as fractions of body mass, from Winter's tables.
//! 2. **The limb solve** — where the elbow or the knee lands, which is
//!    [`garust::twolink`] with the branch named in anatomical terms
//!    ("the elbow goes backwards") instead of as a vector.
//!
//! The lengths are fractions and not millimetres on purpose: every figure
//! that reaches the screen has to move when the person does, or the
//! comparison silently becomes "my model versus your body".

use garust::twolink::{two_link_joint, TwoLinkError};

/// Winter's segment-length fractions of stature.
pub mod largo {
    /// Ankle height above the floor.
    pub const TOBILLO: f64 = 0.039;
    /// Ankle → knee.
    pub const TIBIA: f64 = 0.246;
    /// Knee → hip.
    pub const FEMUR: f64 = 0.245;
    /// L5/S1 → shoulder.
    pub const TORSO: f64 = 0.288;
    /// Shoulder → centre of the grip, arm hanging.
    pub const AGARRE: f64 = 0.372;
    /// Shoulder → head centre of mass, along the trunk axis.
    pub const CABEZA: f64 = 0.130;
    /// How the grip length splits into upper arm and forearm.
    pub const HUMERO_DEL_BRAZO: f64 = 0.52;
}

/// Winter's segment-mass fractions of body mass.
///
/// The paired ones (`PIE`, `TIBIA`, `FEMUR`, `BRAZO`) are **per side**:
/// a two-legged model counts them twice. Getting that wrong is worth a
/// tenth of the body, which is why it is said here and not assumed.
pub mod masa {
    /// One foot.
    pub const PIE: f64 = 0.0145;
    /// One shank.
    pub const TIBIA: f64 = 0.0465;
    /// One thigh.
    pub const FEMUR: f64 = 0.100;
    /// Pelvis.
    pub const PELVIS: f64 = 0.142;
    /// Trunk above L5/S1, without head or arms.
    pub const TRONCO: f64 = 0.355;
    /// Head and neck.
    pub const CABEZA: f64 = 0.081;
    /// One whole arm: upper arm, forearm and hand.
    pub const BRAZO: f64 = 0.050;
}

/// Where a segment's centre of mass sits, as a fraction from its
/// proximal end.
pub mod centro {
    /// Shank, from the knee.
    pub const TIBIA: f64 = 0.433;
    /// Thigh, from the hip.
    pub const FEMUR: f64 = 0.433;
    /// Trunk, from L5/S1.
    pub const TRONCO: f64 = 0.56;
}

/// One person: everything the model needs about a body.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Persona {
    /// Stature, metres.
    pub estatura_m: f64,
    /// Body mass, kilograms.
    pub masa_kg: f64,
}

impl Persona {
    /// L5/S1 → shoulder.
    #[must_use]
    pub fn torso(&self) -> f64 {
        largo::TORSO * self.estatura_m
    }
    /// Shoulder → grip, arm hanging straight.
    #[must_use]
    pub fn brazo(&self) -> f64 {
        largo::AGARRE * self.estatura_m
    }
    /// Shoulder → elbow.
    #[must_use]
    pub fn humero(&self) -> f64 {
        self.brazo() * largo::HUMERO_DEL_BRAZO
    }
    /// Elbow → grip.
    #[must_use]
    pub fn antebrazo(&self) -> f64 {
        self.brazo() * (1.0 - largo::HUMERO_DEL_BRAZO)
    }
    /// Ankle → knee.
    #[must_use]
    pub fn tibia(&self) -> f64 {
        largo::TIBIA * self.estatura_m
    }
    /// Knee → hip.
    #[must_use]
    pub fn femur(&self) -> f64 {
        largo::FEMUR * self.estatura_m
    }
    /// A mass fraction turned into kilograms.
    #[must_use]
    pub fn masa_de(&self, fraccion: f64) -> f64 {
        fraccion * self.masa_kg
    }
    /// Everything hanging off L5/S1: trunk, head and **both** arms.
    ///
    /// This total is the one that made a published figure 11% low, back
    /// when trunk, head and arms were lumped at the trunk's centre of
    /// mass. The total was never the problem; where it acts was.
    #[must_use]
    pub fn masa_sobre_l5(&self) -> f64 {
        self.masa_de(masa::TRONCO + masa::CABEZA + 2.0 * masa::BRAZO)
    }
}

/// Which way a limb bends, in anatomical terms.
///
/// A direction vector says the same thing, and says it in a frame the
/// caller has to get right. These are the two answers a sagittal limb
/// has, named for what they look like.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Doblez {
    /// The joint leads toward `+x` — an elbow in front, a knee forward.
    Adelante,
    /// The joint leads toward `−x` — an elbow tucked back, a hip hinging.
    Atras,
    /// The joint hangs below — a seated row's elbow.
    Abajo,
    /// The joint rides above.
    Arriba,
    /// Any other direction, `[x, y]` in the sagittal plane.
    ///
    /// The four named ones read better and cover most limbs, but they are
    /// not enough and the press is why. At the start of a military press
    /// the bar sits in front of the shoulder at the **same height**, so
    /// the shoulder→hand axis is horizontal and "forwards" names no side
    /// of it — the solve refuses rather than guessing, which is correct
    /// and useless. `Hacia([1.0, -0.3])` is what that lift actually needs.
    Hacia([f64; 2]),
}

impl Doblez {
    fn hint(self) -> [f64; 3] {
        match self {
            Doblez::Adelante => [1.0, 0.0, 0.0],
            Doblez::Atras => [-1.0, 0.0, 0.0],
            Doblez::Abajo => [0.0, -1.0, 0.0],
            Doblez::Arriba => [0.0, 1.0, 0.0],
            Doblez::Hacia([x, y]) => [x, y, 0.0],
        }
    }
}

/// The elbow, given shoulder and hand — sagittal plane, `z = 0`.
///
/// The bend is **given**, never inferred from the geometry. With a hand
/// hanging straight below the shoulder the two solutions are mirror
/// images at the same height, and a rule like "take the higher one" is a
/// coin flip that once drew a rower's elbow backwards in a published
/// reel. [`garust::twolink`] carries that as a claim.
///
/// # Errors
/// Passes through [`TwoLinkError`].
pub fn codo(
    p: &Persona,
    hombro: [f64; 2],
    mano: [f64; 2],
    hacia: Doblez,
) -> Result<[f64; 2], TwoLinkError> {
    let j = two_link_joint(
        [hombro[0], hombro[1], 0.0],
        [mano[0], mano[1], 0.0],
        p.humero(),
        p.antebrazo(),
        hacia.hint(),
    )?;
    Ok([j[0], j[1]])
}

/// The knee, given ankle and hip — sagittal plane, `z = 0`.
///
/// # Errors
/// Passes through [`TwoLinkError`].
pub fn rodilla(
    p: &Persona,
    tobillo: [f64; 2],
    cadera: [f64; 2],
    hacia: Doblez,
) -> Result<[f64; 2], TwoLinkError> {
    let j = two_link_joint(
        [tobillo[0], tobillo[1], 0.0],
        [cadera[0], cadera[1], 0.0],
        p.tibia(),
        p.femur(),
        hacia.hint(),
    )?;
    Ok([j[0], j[1]])
}

#[cfg(test)]
mod tests {
    use super::{codo, masa, Doblez, Persona};

    fn gustavo() -> Persona {
        Persona { estatura_m: 1.75, masa_kg: 80.0 }
    }

    /// The published figures of the row reel come from these fractions.
    #[test]
    fn reproduces_the_published_segment_lengths() {
        let p = gustavo();
        assert!((p.torso() - 0.504).abs() < 1e-9, "L5/S1 → hombro: {}", p.torso());
        assert!((p.brazo() - 0.651).abs() < 1e-9, "hombro → agarre: {}", p.brazo());
        assert!((p.humero() - 0.651 * 0.52).abs() < 1e-9);
        assert!((p.humero() + p.antebrazo() - p.brazo()).abs() < 1e-12, "el brazo es sus dos partes");
    }

    /// The paired fractions count twice, and the total says so.
    #[test]
    fn what_hangs_off_l5_counts_both_arms() {
        let p = gustavo();
        let esperado = (masa::TRONCO + masa::CABEZA + 2.0 * masa::BRAZO) * 80.0;
        assert!((p.masa_sobre_l5() - esperado).abs() < 1e-12);
        assert!((p.masa_sobre_l5() - 42.88).abs() < 1e-9, "{}", p.masa_sobre_l5());
    }

    /// The row's degenerate case, in anatomical terms: hand straight below
    /// the shoulder, and the elbow still goes where it was told.
    #[test]
    fn the_row_elbow_goes_where_it_is_told() {
        let p = gustavo();
        let hombro = [0.436, 1.202];
        let mano = [0.436, 1.202 - p.brazo() * 0.75];
        let atras = codo(&p, hombro, mano, Doblez::Atras).unwrap();
        let adelante = codo(&p, hombro, mano, Doblez::Adelante).unwrap();
        assert!(atras[0] < hombro[0] - 1e-3, "atrás: {atras:?}");
        assert!(adelante[0] > hombro[0] + 1e-3, "adelante: {adelante:?}");
        assert!((atras[1] - adelante[1]).abs() < 1e-12, "espejo, misma altura");
    }

    /// Everything scales with the person: nobody's body is hard-coded.
    #[test]
    fn a_taller_person_has_longer_everything() {
        let bajo = Persona { estatura_m: 1.60, masa_kg: 80.0 };
        let alto = Persona { estatura_m: 1.90, masa_kg: 80.0 };
        assert!(alto.torso() > bajo.torso());
        assert!(alto.femur() > bajo.femur());
        assert!(alto.brazo() > bajo.brazo());
        // la masa no: es fracción de la masa, no de la estatura
        assert!((alto.masa_sobre_l5() - bajo.masa_sobre_l5()).abs() < 1e-12);
    }
}
