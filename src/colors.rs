//TODO: Fix the color mapping
#![allow(dead_code)]

pub const BLACK: u8 = 0;
pub const PINK: u8 = 1;
pub const RED: u8 = 2;
pub const ORANGE: u8 = 3;

pub const ORANGE2: u8 = 4;
pub const BROWN_PALE: u8 = 5;
pub const BROWN: u8 = 6;
pub const YELLOW_PALE: u8 = 7;

pub const YELLOW: u8 = 8;
pub const GREEN_LIME: u8 = 9;
pub const GREEN_LIGHT: u8 = 10;
pub const GREEN: u8 = 11;

pub const GREEN_TURTLE: u8 = 12;
pub const GREEN_PALE: u8 = 13;
pub const TURQUOISE_PALE: u8 = 14;
pub const TURQUOISE: u8 = 15;

pub const BLUE_SKY: u8 = 16;
pub const PURPLE_PALE: u8 = 17;
pub const PURPLE_BLUE: u8 = 18;
pub const PURPLE: u8 = 19;

pub const BLUE_SKY_DARK: u8 = 20;
// pub const YELLOW_AMBER_BRIGHT: u8 = 21;
// pub const YELLOW_LOW: u8 = 22;
// pub const YELLOW: u8 = 23;
//
// pub const YELLOW_BRIGHT: u8 = 24;
// pub const YELLOW_LIME_LOW: u8 = 25;
// pub const YELLOW_LIME: u8 = 26;
// pub const YELLOW_LIME_BRIGHT: u8 = 27;
//
// pub const LIME_YELLOW_LOW: u8 = 28;
// pub const LIME_YELLOW: u8 = 29;
// pub const LIME_YELLOW_BRIGHT: u8 = 30;
// pub const LIME_LOW: u8 = 31;
//
// pub const LIME: u8 = 32;
// pub const LIME_BRIGHT: u8 = 33;
// pub const LIME_GREEN_LOW: u8 = 34;
// pub const LIME_GREEN: u8 = 35;
//
// pub const LIME_GREEN_BRIGHT: u8 = 36;
// pub const GREEN_LIME_LOW: u8 = 37;
// pub const GREEN_LIME: u8 = 38;
// pub const GREEN_LIME_BRIGHT: u8 = 39;
//
// pub const GREEN_LOW: u8 = 40;
// pub const GREEN: u8 = 41;
// pub const GREEN_BRIGHT: u8 = 42;
// pub const GREEN_SPRING_LOW: u8 = 43;
//
// pub const GREEN_SPRING: u8 = 44;
// pub const GREEN_SPRING_BRIGHT: u8 = 45;
// pub const SPRING_GREEN_LOW: u8 = 46;
// pub const SPRING_GREEN: u8 = 47;
//
// pub const SPRING_GREEN_BRIGHT: u8 = 48;
// pub const SPRING_LOW: u8 = 49;
// pub const SPRING: u8 = 50;
// pub const SPRING_BRIGHT: u8 = 51;
//
// pub const SPRING_CYAN_LOW: u8 = 52;
// pub const SPRING_CYAN: u8 = 53;
// pub const SPRING_CYAN_BRIGHT: u8 = 54;
// pub const CYAN_SPRING_LOW: u8 = 55;
//
// pub const CYAN_SPRING: u8 = 56;
// pub const CYAN_SPRING_BRIGHT: u8 = 57;
// pub const CYAN_LOW: u8 = 58;
// pub const CYAN: u8 = 59;
//
// pub const CYAN_BRIGHT: u8 = 60;
// pub const CYAN_AZURE_LOW: u8 = 61;
// pub const CYAN_AZURE: u8 = 62;
// pub const CYAN_AZURE_BRIGHT: u8 = 63;
//
// pub const AZURE_CYAN_LOW: u8 = 64;
// pub const AZURE_CYAN: u8 = 65;
// pub const AZURE_CYAN_BRIGHT: u8 = 66;
// pub const AZURE_LOW: u8 = 67;
//
// pub const AZURE: u8 = 68;
// pub const AZURE_BRIGHT: u8 = 69;
// pub const AZURE_BLUE_LOW: u8 = 70;
// pub const AZURE_BLUE: u8 = 71;
//
// pub const AZURE_BLUE_BRIGHT: u8 = 72;
// pub const BLUE_AZURE_LOW: u8 = 73;
// pub const BLUE_AZURE: u8 = 74;
// pub const BLUE_AZURE_BRIGHT: u8 = 75;
//
// pub const BLUE_LOW: u8 = 76;
// pub const BLUE: u8 = 77;
// pub const BLUE_BRIGHT: u8 = 78;
// pub const BLUE_VIOLET_LOW: u8 = 79;
//
// pub const BLUE_VIOLET: u8 = 80;
// pub const BLUE_VIOLET_BRIGHT: u8 = 81;
// pub const VIOLET_BLUE_LOW: u8 = 82;
// pub const VIOLET_BLUE: u8 = 83;
//
// pub const VIOLET_BLUE_BRIGHT: u8 = 84;
// pub const VIOLET_LOW: u8 = 85;
// pub const VIOLET: u8 = 86;
// pub const VIOLET_BRIGHT: u8 = 87;
//
// pub const VIOLET_MAGENTA_LOW: u8 = 88;
// pub const VIOLET_MAGENTA: u8 = 89;
// pub const VIOLET_MAGENTA_BRIGHT: u8 = 90;
// pub const MAGENTA_VIOLET_LOW: u8 = 91;
//
// pub const MAGENTA_VIOLET: u8 = 92;
// pub const MAGENTA_VIOLET_BRIGHT: u8 = 93;
// pub const MAGENTA_LOW: u8 = 94;
// pub const MAGENTA: u8 = 95;
//
// pub const MAGENTA_BRIGHT: u8 = 96;
// pub const MAGENTA_PINK_LOW: u8 = 97;
// pub const MAGENTA_PINK: u8 = 98;
// pub const MAGENTA_PINK_BRIGHT: u8 = 99;
//
// pub const PINK_MAGENTA_LOW: u8 = 100;
// pub const PINK_MAGENTA: u8 = 101;
// pub const PINK_MAGENTA_BRIGHT: u8 = 102;
// pub const PINK_LOW: u8 = 103;
//
// pub const PINK: u8 = 104;
// pub const PINK_BRIGHT: u8 = 105;
// pub const PINK_RED_LOW: u8 = 106;
// pub const PINK_RED: u8 = 107;
//
// pub const PINK_RED_BRIGHT: u8 = 108;
// pub const RED_PINK_LOW: u8 = 109;
// pub const RED_PINK: u8 = 110;
// pub const RED_PINK_BRIGHT: u8 = 111;
//
// pub const RED_LOW: u8 = 112;
// pub const RED: u8 = 113;
// pub const RED_BRIGHT: u8 = 114;
// pub const WARM_WHITE_LOW: u8 = 115;
//
// pub const WARM_WHITE: u8 = 116;
// pub const WARM_WHITE_BRIGHT: u8 = 117;
// pub const WHITE_LOW: u8 = 118;
// pub const WHITE_BRIGHT: u8 = 119;
//
// pub const ORANGE_LOW: u8 = 120;
// pub const ORANGE: u8 = 121;
// pub const ORANGE_BRIGHT: u8 = 122;
// pub const YELLOW_PALE: u8 = 123;
// pub const LIME_PALE: u8 = 124;
// pub const GREEN_PALE: u8 = 125;
// pub const CYAN_PALE: u8 = 126;
// pub const BLUE_PALE: u8 = 127;

