use bsp::hal::time::Hertz;
use defmt::Format;

macro_rules! tones {
    (
        $($key:ident: $freq:expr),+
    ) => {
        #[derive(Format, Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Tone {
            $(
                $key,
            )*
        }

        impl Tone {
            pub fn hz(&self) -> Hertz {
                match *self {
                    $(Tone::$key => Hertz($freq),)*
                }
            }

            pub fn freq(&self) -> u32 {
                match *self {
                    $(Tone::$key => $freq,)*
                }
            }
        }
    };
}

tones!(
    REST: 0,
    // C
    C1: 33,
    C2: 65,
    C3: 131,
    C4: 262,
    C5: 523,
    C6: 1047,
    C7: 2093,
    C8: 4186,
    C9: 8372,
    // C♯/D♭
    CS1: 35,
    CS2: 69,
    CS3: 139,
    CS4: 277,
    CS5: 554,
    CS6: 1109,
    CS7: 2218,
    CS8: 4435,
    CS9: 8869,
    // D
    D1: 37,
    D2: 73,
    D3: 147,
    D4: 294,
    D5: 587,
    D6: 1175,
    D7: 2349,
    D8: 4699,
    D9: 9397,
    // D♯/E♭
    DS1: 39,
    DS2: 78,
    DS3: 156,
    DS4: 311,
    DS5: 622,
    DS6: 1245,
    DS7: 2489,
    DS8: 4978,
    DS9: 9956,
    // E
    E1: 41,
    E2: 82,
    E3: 165,
    E4: 330,
    E5: 659,
    E6: 1319,
    E7: 2637,
    E8: 5274,
    E9: 10548,
    // F
    F1: 44,
    F2: 87,
    F3: 175,
    F4: 349,
    F5: 698,
    F6: 1397,
    F7: 2794,
    F8: 5588,
    F9: 11175,
    // F♯/G♭
    FS1: 46,
    FS2: 92,
    FS3: 185,
    FS4: 370,
    FS5: 740,
    FS6: 1480,
    FS7: 2960,
    FS8: 5920,
    FS9: 11840,
    // G
    G1: 49,
    G2: 98,
    G3: 196,
    G4: 392,
    G5: 784,
    G6: 1568,
    G7: 3136,
    G8: 6272,
    G9: 12544,
    // G♯/A♭
    GS1: 52,
    GS2: 104,
    GS3: 208,
    GS4: 415,
    GS5: 831,
    GS6: 1661,
    GS7: 3322,
    GS8: 6645,
    GS9: 13290,
    // A
    A1: 55,
    A2: 110,
    A3: 220,
    A4: 440,
    A5: 880,
    A6: 1760,
    A7: 3520,
    A8: 7040,
    A9: 14080,
    // A♯/B♭
    AS1: 58,
    AS2: 117,
    AS3: 233,
    AS4: 466,
    AS5: 932,
    AS6: 1865,
    AS7: 3729,
    AS8: 7459,
    AS9: 14917,
    // B
    B1: 62,
    B2: 123,
    B3: 247,
    B4: 494,
    B5: 988,
    B6: 1976,
    B7: 3951,
    B8: 7902,
    B9: 15804
);
