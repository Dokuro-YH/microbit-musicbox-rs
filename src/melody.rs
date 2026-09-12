use defmt::Format;

use crate::tone::Tone;

#[derive(Format, Debug)]
pub struct Melody {
    whole_note_delay_ms: u32,
    notes: &'static [(Tone, i8)],
}

impl Melody {
    pub fn get(&self, pos: usize) -> Option<(Tone, u32)> {
        self.notes.get(pos).cloned().map(|(note, div)| {
            let dotted = div <= 0;
            let div = div.abs() as f32;
            let delay_ms = self.whole_note_delay_ms as f32 / div;
            (note, if dotted { delay_ms * 1.5 } else { delay_ms } as u32)
        })
    }

    pub fn len(&self) -> usize {
        self.notes.len()
    }
}

macro_rules! melody {
    (
        name = $name:ident,
        tempo = $tempo:expr,
        beat = $beat:expr,
        $([$($note:ident: $duration:expr),*]),*
    ) => {
        pub const $name: Melody = Melody {
            whole_note_delay_ms: (60000 * $beat) / $tempo,
            notes: &[
                $(
                    $((Tone::$note, $duration),)*
                )*
            ]
        };
    };
}

// Happy birthday
// https://musescore.com/user/8221/scores/26906
melody!(
    name = HAPPY_BIRTHDAY, tempo = 140, beat = 4,
    [C4:4, C4:8, D4:-4, C4:-4, F4:-4, E4:-2],
    [C4:4, C4:8, D4:-4, C4:-4, G4:-4, F4:-2],
    [C4:4, C4:8, C5:-4, A4:-4, F4:-4, E4:-4, D4:-4],
    [AS4:4, AS4:8, A4:-4, F4:-4, G4:-4, F4:-2]
);

// We Wish You a Merry Christmas
// https://musescore.com/user/6208766/scores/1497501
melody!(
    name = MERRY_CHRISTMAS, tempo = 140, beat = 4,
    [C5:4, //1
    F5:4, F5:8, G5:8, F5:8, E5:8,
    D5:4, D5:4, D5:4,
    G5:4, G5:8, A5:8, G5:8, F5:8,
    E5:4, C5:4, C5:4,
    A5:4, A5:8, AS5:8, A5:8, G5:8,
    F5:4, D5:4, C5:8, C5:8,
    D5:4, G5:4, E5:4],

    [F5:2, C5:4, //8
    F5:4, F5:8, G5:8, F5:8, E5:8,
    D5:4, D5:4, D5:4,
    G5:4, G5:8, A5:8, G5:8, F5:8,
    E5:4, C5:4, C5:4,
    A5:4, A5:8, AS5:8, A5:8, G5:8,
    F5:4, D5:4, C5:8, C5:8,
    D5:4, G5:4, E5:4,
    F5:2, C5:4],

    [F5:4, F5:4, F5:4,//17
    E5:2, E5:4,
    F5:4, E5:4, D5:4,
    C5:2, A5:4,
    AS5:4, A5:4, G5:4,
    C6:4, C5:4, C5:8, C5:8,
    D5:4, G5:4, E5:4,
    F5:2, C5:4,
    F5:4, F5:8, G5:8, F5:8, E5:8,
    D5:4, D5:4, D5:4],

    [G5:4, G5:8, A5:8, G5:8, F5:8, //27
    E5:4, C5:4, C5:4,
    A5:4, A5:8, AS5:8, A5:8, G5:8,
    F5:4, D5:4, C5:8, C5:8,
    D5:4, G5:4, E5:4,
    F5:2, C5:4,
    F5:4, F5:4, F5:4,
    E5:2, E5:4,
    F5:4, E5:4, D5:4],

    [C5:2, A5:4,//36
    AS5:4, A5:4, G5:4,
    C6:4, C5:4, C5:8, C5:8,
    D5:4, G5:4, E5:4,
    F5:2, C5:4,
    F5:4, F5:8, G5:8, F5:8, E5:8,
    D5:4, D5:4, D5:4,
    G5:4, G5:8, A5:8, G5:8, F5:8,
    E5:4, C5:4, C5:4],

    [A5:4, A5:8, AS5:8, A5:8, G5:8,//45
    F5:4, D5:4, C5:8, C5:8,
    D5:4, G5:4, E5:4,
    F5:2, C5:4,
    F5:4, F5:8, G5:8, F5:8, E5:8,
    D5:4, D5:4, D5:4,
    G5:4, G5:8, A5:8, G5:8, F5:8,
    E5:4, C5:4, C5:4],

    [A5:4, A5:8, AS5:8, A5:8, G5:8, //53
    F5:4, D5:4, C5:8, C5:8,
    D5:4, G5:4, E5:4,
    F5:2, REST:4]
);

melody!(
    name = SUPER_MARIOBROS, tempo = 200, beat = 4,
    [E5:8, E5:8, REST:8, E5:8, REST:8, C5:8, E5:8, //1
    G5:4, REST:4, G4:8, REST:4],

    [C5:-4, G4:8, REST:4, E4:-4, // 3
    A4:4, B4:4, AS4:8, A4:4,
    G4:-8, E5:-8, G5:-8, A5:4, F5:8, G5:8,
    REST:8, E5:4,C5:8, D5:8, B4:-4],

    [C5:-4, G4:8, REST:4, E4:-4, // repeats from 3
    A4:4, B4:4, AS4:8, A4:4,
    G4:-8, E5:-8, G5:-8, A5:4, F5:8, G5:8,
    REST:8, E5:4,C5:8, D5:8, B4:-4],

    [REST:4, G5:8, FS5:8, F5:8, DS5:4, E5:8,//7
    REST:8, GS4:8, A4:8, C4:8, REST:8, A4:8, C5:8, D5:8,
    REST:4, DS5:4, REST:8, D5:-4,
    C5:2, REST:2],

    [REST:4, G5:8, FS5:8, F5:8, DS5:4, E5:8,//repeats from 7
    REST:8, GS4:8, A4:8, C4:8, REST:8, A4:8, C5:8, D5:8,
    REST:4, DS5:4, REST:8, D5:-4,
    C5:2, REST:2],

    [C5:8, C5:4, C5:8, REST:8, C5:8, D5:4,//11
    E5:8, C5:4, A4:8, G4:2],

    [C5:8, C5:4, C5:8, REST:8, C5:8, D5:8, E5:8,//13
    REST:1,
    C5:8, C5:4, C5:8, REST:8, C5:8, D5:4,
    E5:8, C5:4, A4:8, G4:2,
    E5:8, E5:8, REST:8, E5:8, REST:8, C5:8, E5:4,
    G5:4, REST:4, G4:4, REST:4],

    [C5:-4, G4:8, REST:4, E4:-4, // 19
    A4:4, B4:4, AS4:8, A4:4,
    G4:-8, E5:-8, G5:-8, A5:4, F5:8, G5:8,
    REST:8, E5:4, C5:8, D5:8, B4:-4],

    [C5:-4, G4:8, REST:4, E4:-4, // repeats from 19
    A4:4, B4:4, AS4:8, A4:4,
    G4:-8, E5:-8, G5:-8, A5:4, F5:8, G5:8,
    REST:8, E5:4, C5:8, D5:8, B4:-4],

    [E5:8, C5:4, G4:8, REST:4, GS4:4,//23
    A4:8, F5:4, F5:8, A4:2,
    D5:-8, A5:-8, A5:-8, A5:-8, G5:-8, F5:-8],

    [E5:8, C5:4, A4:8, G4:2, //26
    E5:8, C5:4, G4:8, REST:4, GS4:4,
    A4:8, F5:4, F5:8, A4:2,
    B4:8, F5:4, F5:8, F5:-8, E5:-8, D5:-8,
    C5:8, E4:4, E4:8, C4:2],

    [E5:8, C5:4, G4:8, REST:4, GS4:4,//repeats from 23
    A4:8, F5:4, F5:8, A4:2,
    D5:-8, A5:-8, A5:-8, A5:-8, G5:-8, F5:-8],

    [E5:8, C5:4, A4:8, G4:2, //26
    E5:8, C5:4, G4:8, REST:4, GS4:4,
    A4:8, F5:4, F5:8, A4:2,
    B4:8, F5:4, F5:8, F5:-8, E5:-8, D5:-8,
    C5:8, E4:4, E4:8, C4:2,
    C5:8, C5:4, C5:8, REST:8, C5:8, D5:8, E5:8,
    REST:1],

    [C5:8, C5:4, C5:8, REST:8, C5:8, D5:4, //33
    E5:8, C5:4, A4:8, G4:2,
    E5:8, E5:8, REST:8, E5:8, REST:8, C5:8, E5:4,
    G5:4, REST:4, G4:4, REST:4,
    E5:8, C5:4, G4:8, REST:4, GS4:4,
    A4:8, F5:4, F5:8, A4:2,
    D5:-8, A5:-8, A5:-8, A5:-8, G5:-8, F5:-8],

    [E5:8, C5:4, A4:8, G4:2, //40
    E5:8, C5:4, G4:8, REST:4, GS4:4,
    A4:8, F5:4, F5:8, A4:2,
    B4:8, F5:4, F5:8, F5:-8, E5:-8, D5:-8,
    C5:8, E4:4, E4:8, C4:2],

    //game over sound
    [C5:-4, G4:-4, E4:4, //45
    A4:-8, B4:-8, A4:-8, GS4:-8, AS4:-8, GS4:-8,
    G4:8, D4:8, E4:-2]
);

melody!(
    name = GAME_OF_THRONES, tempo = 85, beat = 4,
    [G4:8, C4:8, DS4:16, F4:16, G4:8, C4:8, DS4:16, F4:16, //1
    G4:8, C4:8, DS4:16, F4:16, G4:8, C4:8, DS4:16, F4:16,
    G4:8, C4:8, E4:16, F4:16, G4:8, C4:8, E4:16, F4:16,
    G4:8, C4:8, E4:16, F4:16, G4:8, C4:8, E4:16, F4:16],

    [G4:-4, C4:-4],//5

    [DS4:16, F4:16, G4:4, C4:4, DS4:16, F4:16], //6

    [D4:-1, //7 and 8
    F4:-4, AS3:-4,
    DS4:16, D4:16, F4:4, AS3:-4],

    [DS4:16, D4:16, C4:-1], //11 and 12

    //repeats from 5
    [G4:-4, C4:-4],//5

    [DS4:16, F4:16, G4:4, C4:4, DS4:16, F4:16], //6

    [D4:-1, //7 and 8
    F4:-4, AS3:-4,
    DS4:16, D4:16, F4:4, AS3:-4],

    [DS4:16, D4:16, C4:-1, //11 and 12
    G4:-4, C4:-4,
    DS4:16, F4:16, G4:4,  C4:4, DS4:16, F4:16],

    [D4:-2,//15
    F4:-4, AS3:-4,
    D4:-8, DS4:-8, D4:-8, AS3:-8,
    C4:-1,
    C5:-2,
    AS4:-2,
    C4:-2,
    G4:-2,
    DS4:-2,
    DS4:-4, F4:-4,
    G4:-1],

    [C5:-2,//28
    AS4:-2,
    C4:-2,
    G4:-2,
    DS4:-2,
    DS4:-4, D4:-4,
    C5:8, G4:8, GS4:16, AS4:16, C5:8, G4:8, GS4:16, AS4:16,
    C5:8, G4:8, GS4:16, AS4:16, C5:8, G4:8, GS4:16, AS4:16,

    REST:4, GS5:16, AS5:16, C6:8, G5:8, GS5:16, AS5:16,
    C6:8, G5:16, GS5:16, AS5:16, C6:8, G5:8, GS5:16, AS5:16]
);
