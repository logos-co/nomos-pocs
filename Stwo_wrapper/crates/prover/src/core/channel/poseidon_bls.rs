use std::iter;

use ark_bls12_381::Fr as BlsFr;
use ark_ff::{BigInteger, Field, PrimeField};
use crypto_bigint::{Encoding, NonZero, U256};

use super::{Channel, ChannelTime};
use crate::core::fields::m31::BaseField;
use crate::core::fields::qm31::SecureField;
use crate::core::fields::secure_column::SECURE_EXTENSION_DEGREE;

pub const BYTES_PER_FELT252: usize = 32;
pub const FELTS_PER_HASH: usize = 8;

//Optimize constant to be real constants (no conversion) and merge duplicated code in VCS poseidono
fn poseidon_comp_consts(idx: usize) -> BlsFr {
    match idx {
        0 => BlsFr::from_be_bytes_mod_order(&[
            111, 0, 122, 85, 17, 86, 179, 164, 73, 228, 73, 54, 183, 192, 147, 100, 74, 14, 211,
            63, 51, 234, 204, 198, 40, 233, 66, 232, 54, 193, 168, 117,
        ]),
        1 => BlsFr::from_be_bytes_mod_order(&[
            54, 13, 116, 112, 97, 30, 71, 61, 53, 63, 98, 143, 118, 209, 16, 243, 78, 113, 22, 47,
            49, 0, 59, 112, 87, 83, 140, 37, 150, 66, 99, 3,
        ]),
        2 => BlsFr::from_be_bytes_mod_order(&[
            75, 95, 236, 58, 160, 115, 223, 68, 1, 144, 145, 240, 7, 164, 76, 169, 150, 72, 73,
            101, 247, 3, 109, 206, 62, 157, 9, 119, 237, 205, 192, 246,
        ]),
        3 => BlsFr::from_be_bytes_mod_order(&[
            103, 207, 24, 104, 175, 99, 150, 192, 184, 76, 206, 113, 94, 83, 159, 132, 158, 6, 205,
            28, 56, 58, 197, 176, 97, 0, 199, 107, 204, 151, 58, 17,
        ]),
        4 => BlsFr::from_be_bytes_mod_order(&[
            85, 93, 180, 209, 220, 237, 129, 159, 93, 61, 231, 15, 222, 131, 241, 199, 211, 232,
            201, 137, 104, 229, 22, 162, 58, 119, 26, 92, 156, 130, 87, 170,
        ]),
        5 => BlsFr::from_be_bytes_mod_order(&[
            43, 171, 148, 215, 174, 34, 45, 19, 93, 195, 198, 197, 254, 191, 170, 49, 73, 8, 172,
            47, 18, 235, 224, 111, 189, 183, 66, 19, 191, 99, 24, 139,
        ]),
        6 => BlsFr::from_be_bytes_mod_order(&[
            102, 244, 75, 229, 41, 102, 130, 196, 250, 120, 130, 121, 157, 109, 208, 73, 182, 215,
            210, 201, 80, 204, 249, 140, 242, 229, 13, 109, 30, 187, 119, 194,
        ]),
        7 => BlsFr::from_be_bytes_mod_order(&[
            21, 12, 147, 254, 246, 82, 251, 28, 43, 240, 62, 26, 41, 170, 135, 31, 239, 119, 231,
            215, 54, 118, 108, 93, 9, 57, 217, 39, 83, 204, 93, 200,
        ]),
        8 => BlsFr::from_be_bytes_mod_order(&[
            50, 112, 102, 30, 104, 146, 139, 58, 149, 93, 85, 219, 86, 220, 87, 193, 3, 204, 10,
            96, 20, 30, 137, 78, 20, 37, 157, 206, 83, 119, 130, 178,
        ]),
        9 => BlsFr::from_be_bytes_mod_order(&[
            7, 63, 17, 111, 4, 18, 46, 37, 160, 183, 175, 228, 226, 5, 114, 153, 180, 7, 195, 112,
            242, 181, 161, 204, 206, 159, 185, 255, 195, 69, 175, 179,
        ]),
        10 => BlsFr::from_be_bytes_mod_order(&[
            64, 159, 218, 34, 85, 140, 254, 77, 61, 216, 220, 226, 79, 105, 231, 111, 140, 42, 174,
            177, 221, 15, 9, 214, 94, 101, 76, 113, 243, 42, 162, 63,
        ]),
        11 => BlsFr::from_be_bytes_mod_order(&[
            42, 50, 236, 92, 78, 229, 177, 131, 122, 255, 208, 156, 31, 83, 245, 253, 85, 201, 205,
            32, 97, 174, 147, 202, 142, 186, 215, 111, 199, 21, 84, 216,
        ]),
        12 => BlsFr::from_be_bytes_mod_order(&[
            88, 72, 235, 235, 89, 35, 233, 37, 85, 183, 18, 79, 255, 186, 93, 107, 213, 113, 198,
            249, 132, 25, 94, 185, 207, 211, 163, 232, 235, 85, 177, 212,
        ]),
        13 => BlsFr::from_be_bytes_mod_order(&[
            39, 3, 38, 238, 3, 157, 241, 158, 101, 30, 44, 252, 116, 6, 40, 202, 99, 77, 36, 252,
            110, 37, 89, 242, 45, 140, 203, 226, 146, 239, 238, 173,
        ]),
        14 => BlsFr::from_be_bytes_mod_order(&[
            39, 198, 100, 42, 198, 51, 188, 102, 220, 16, 15, 231, 252, 250, 84, 145, 138, 248,
            149, 188, 224, 18, 241, 130, 160, 104, 252, 55, 193, 130, 226, 116,
        ]),
        15 => BlsFr::from_be_bytes_mod_order(&[
            27, 223, 216, 176, 20, 1, 199, 10, 210, 127, 87, 57, 105, 137, 18, 157, 113, 14, 31,
            182, 171, 151, 106, 69, 156, 161, 134, 130, 226, 109, 127, 249,
        ]),
        16 => BlsFr::from_be_bytes_mod_order(&[
            73, 27, 155, 166, 152, 59, 207, 159, 5, 254, 71, 148, 173, 180, 74, 48, 135, 155, 248,
            40, 150, 98, 225, 245, 125, 144, 246, 114, 65, 78, 138, 74,
        ]),
        17 => BlsFr::from_be_bytes_mod_order(&[
            22, 42, 20, 198, 47, 154, 137, 184, 20, 185, 214, 169, 200, 77, 214, 120, 244, 246,
            251, 63, 144, 84, 211, 115, 200, 50, 216, 36, 38, 26, 53, 234,
        ]),
        18 => BlsFr::from_be_bytes_mod_order(&[
            45, 25, 62, 15, 118, 222, 88, 107, 42, 246, 247, 158, 49, 39, 254, 234, 172, 10, 31,
            199, 30, 44, 240, 192, 247, 152, 36, 102, 123, 91, 107, 236,
        ]),
        19 => BlsFr::from_be_bytes_mod_order(&[
            70, 239, 216, 169, 162, 98, 214, 216, 253, 201, 202, 92, 4, 176, 152, 47, 36, 221, 204,
            110, 152, 99, 136, 90, 106, 115, 42, 57, 6, 160, 123, 149,
        ]),
        20 => BlsFr::from_be_bytes_mod_order(&[
            80, 151, 23, 224, 194, 0, 227, 201, 45, 141, 202, 41, 115, 179, 219, 69, 240, 120, 130,
            148, 53, 26, 208, 122, 231, 92, 187, 120, 6, 147, 167, 152,
        ]),
        21 => BlsFr::from_be_bytes_mod_order(&[
            114, 153, 178, 132, 100, 168, 201, 79, 185, 212, 223, 97, 56, 15, 57, 192, 220, 169,
            194, 192, 20, 17, 135, 137, 226, 39, 37, 40, 32, 240, 27, 252,
        ]),
        22 => BlsFr::from_be_bytes_mod_order(&[
            4, 76, 163, 204, 74, 133, 215, 59, 129, 105, 110, 241, 16, 78, 103, 79, 79, 239, 248,
            41, 132, 153, 15, 248, 93, 11, 245, 141, 200, 164, 170, 148,
        ]),
        23 => BlsFr::from_be_bytes_mod_order(&[
            28, 186, 242, 179, 113, 218, 198, 168, 29, 4, 83, 65, 109, 62, 35, 92, 184, 217, 226,
            212, 243, 20, 244, 111, 97, 152, 120, 95, 12, 214, 185, 175,
        ]),
        24 => BlsFr::from_be_bytes_mod_order(&[
            29, 91, 39, 119, 105, 44, 32, 91, 14, 108, 73, 208, 97, 182, 181, 244, 41, 60, 74, 176,
            56, 253, 187, 220, 52, 62, 7, 97, 15, 63, 237, 229,
        ]),
        25 => BlsFr::from_be_bytes_mod_order(&[
            86, 174, 124, 122, 82, 147, 189, 194, 62, 133, 225, 105, 140, 129, 199, 127, 138, 216,
            140, 75, 51, 165, 120, 4, 55, 173, 4, 124, 110, 219, 89, 186,
        ]),
        26 => BlsFr::from_be_bytes_mod_order(&[
            46, 155, 219, 186, 61, 211, 75, 255, 170, 48, 83, 91, 221, 116, 154, 126, 6, 169, 173,
            176, 193, 230, 249, 98, 246, 14, 151, 27, 141, 115, 176, 79,
        ]),
        27 => BlsFr::from_be_bytes_mod_order(&[
            45, 225, 24, 134, 177, 128, 17, 202, 139, 213, 186, 227, 105, 105, 41, 159, 222, 64,
            251, 226, 109, 4, 123, 5, 3, 90, 19, 102, 31, 34, 65, 139,
        ]),
        28 => BlsFr::from_be_bytes_mod_order(&[
            46, 7, 222, 23, 128, 184, 167, 13, 13, 91, 74, 63, 24, 65, 220, 216, 42, 185, 57, 92,
            68, 155, 233, 71, 188, 153, 136, 132, 186, 150, 167, 33,
        ]),
        29 => BlsFr::from_be_bytes_mod_order(&[
            15, 105, 241, 133, 77, 32, 202, 12, 187, 219, 99, 219, 213, 45, 173, 22, 37, 4, 64,
            169, 157, 107, 138, 243, 130, 94, 76, 43, 183, 73, 37, 202,
        ]),
        30 => BlsFr::from_be_bytes_mod_order(&[
            93, 201, 135, 49, 142, 110, 89, 193, 175, 184, 123, 101, 93, 213, 140, 193, 210, 46,
            81, 58, 5, 131, 140, 212, 88, 93, 4, 177, 53, 185, 87, 202,
        ]),
        31 => BlsFr::from_be_bytes_mod_order(&[
            72, 183, 37, 117, 133, 113, 201, 223, 108, 1, 220, 99, 154, 133, 240, 114, 151, 105,
            107, 27, 182, 120, 99, 58, 41, 220, 145, 222, 149, 239, 83, 246,
        ]),
        32 => BlsFr::from_be_bytes_mod_order(&[
            94, 86, 94, 8, 192, 130, 16, 153, 37, 107, 86, 73, 14, 174, 225, 213, 115, 175, 209,
            11, 182, 209, 125, 19, 202, 78, 92, 97, 27, 42, 55, 24,
        ]),
        33 => BlsFr::from_be_bytes_mod_order(&[
            46, 177, 178, 84, 23, 254, 23, 103, 13, 19, 93, 198, 57, 251, 9, 164, 108, 229, 17, 53,
            7, 249, 109, 233, 129, 108, 5, 148, 34, 220, 112, 94,
        ]),
        34 => BlsFr::from_be_bytes_mod_order(&[
            17, 92, 208, 160, 100, 60, 251, 152, 140, 36, 203, 68, 195, 250, 180, 138, 255, 54,
            198, 97, 210, 108, 196, 45, 184, 177, 189, 244, 149, 59, 216, 44,
        ]),
        35 => BlsFr::from_be_bytes_mod_order(&[
            38, 202, 41, 63, 123, 44, 70, 45, 6, 109, 115, 120, 185, 153, 134, 139, 187, 87, 221,
            241, 78, 15, 149, 138, 222, 128, 22, 18, 49, 29, 4, 205,
        ]),
        36 => BlsFr::from_be_bytes_mod_order(&[
            65, 71, 64, 13, 142, 26, 172, 207, 49, 26, 107, 91, 118, 32, 17, 171, 62, 69, 50, 110,
            77, 75, 157, 226, 105, 146, 129, 107, 153, 197, 40, 172,
        ]),
        37 => BlsFr::from_be_bytes_mod_order(&[
            107, 13, 183, 220, 204, 75, 161, 178, 104, 246, 189, 204, 77, 55, 40, 72, 212, 167, 41,
            118, 194, 104, 234, 48, 81, 154, 47, 115, 230, 219, 77, 85,
        ]),
        38 => BlsFr::from_be_bytes_mod_order(&[
            23, 191, 27, 147, 196, 199, 224, 26, 42, 131, 10, 161, 98, 65, 44, 217, 15, 22, 11,
            249, 247, 30, 150, 127, 245, 32, 157, 20, 178, 72, 32, 202,
        ]),
        39 => BlsFr::from_be_bytes_mod_order(&[
            75, 67, 28, 217, 239, 237, 188, 148, 207, 30, 202, 111, 158, 156, 24, 57, 208, 230,
            106, 139, 255, 168, 200, 70, 76, 172, 129, 163, 157, 60, 248, 241,
        ]),
        40 => BlsFr::from_be_bytes_mod_order(&[
            53, 180, 26, 122, 196, 243, 197, 113, 162, 79, 132, 86, 54, 156, 133, 223, 224, 60, 3,
            84, 189, 140, 253, 56, 5, 200, 111, 46, 125, 194, 147, 197,
        ]),
        41 => BlsFr::from_be_bytes_mod_order(&[
            59, 20, 128, 8, 5, 35, 196, 57, 67, 89, 39, 153, 72, 73, 190, 169, 100, 225, 77, 59,
            235, 45, 221, 222, 114, 172, 21, 106, 244, 53, 208, 158,
        ]),
        42 => BlsFr::from_be_bytes_mod_order(&[
            44, 198, 129, 0, 49, 220, 27, 13, 73, 80, 133, 109, 201, 7, 213, 117, 8, 226, 134, 68,
            42, 45, 62, 178, 39, 22, 24, 216, 116, 177, 76, 109,
        ]),
        43 => BlsFr::from_be_bytes_mod_order(&[
            111, 65, 65, 200, 64, 28, 90, 57, 91, 166, 121, 14, 253, 113, 199, 12, 4, 175, 234, 6,
            195, 201, 40, 38, 188, 171, 221, 92, 181, 71, 125, 81,
        ]),
        44 => BlsFr::from_be_bytes_mod_order(&[
            37, 189, 187, 237, 161, 189, 232, 193, 5, 150, 24, 226, 175, 210, 239, 153, 158, 81,
            122, 169, 59, 120, 52, 29, 145, 243, 24, 192, 159, 12, 181, 102,
        ]),
        45 => BlsFr::from_be_bytes_mod_order(&[
            57, 42, 74, 135, 88, 224, 110, 232, 185, 95, 51, 194, 93, 222, 138, 192, 42, 94, 208,
            162, 123, 97, 146, 108, 198, 49, 52, 135, 7, 63, 127, 123,
        ]),
        46 => BlsFr::from_be_bytes_mod_order(&[
            39, 42, 85, 135, 138, 8, 68, 43, 154, 166, 17, 31, 77, 224, 9, 72, 94, 106, 111, 209,
            93, 184, 147, 101, 231, 187, 206, 240, 46, 181, 134, 108,
        ]),
        47 => BlsFr::from_be_bytes_mod_order(&[
            99, 30, 193, 214, 210, 141, 217, 232, 36, 238, 137, 163, 7, 48, 174, 247, 171, 70, 58,
            207, 201, 209, 132, 179, 85, 170, 5, 253, 105, 56, 234, 181,
        ]),
        48 => BlsFr::from_be_bytes_mod_order(&[
            78, 182, 253, 161, 15, 208, 251, 222, 2, 199, 68, 155, 251, 221, 195, 91, 205, 130, 37,
            231, 229, 195, 131, 58, 8, 24, 161, 0, 64, 157, 198, 242,
        ]),
        49 => BlsFr::from_be_bytes_mod_order(&[
            45, 91, 48, 139, 12, 240, 44, 223, 239, 161, 60, 78, 96, 226, 98, 57, 166, 235, 186, 1,
            22, 148, 221, 18, 155, 146, 91, 60, 91, 33, 224, 226,
        ]),
        50 => BlsFr::from_be_bytes_mod_order(&[
            22, 84, 159, 198, 175, 47, 59, 114, 221, 93, 41, 61, 114, 226, 229, 242, 68, 223, 244,
            47, 24, 180, 108, 86, 239, 56, 197, 124, 49, 22, 115, 172,
        ]),
        51 => BlsFr::from_be_bytes_mod_order(&[
            66, 51, 38, 119, 255, 53, 156, 94, 141, 184, 54, 217, 245, 251, 84, 130, 46, 57, 189,
            94, 34, 52, 11, 185, 186, 151, 91, 161, 169, 43, 227, 130,
        ]),
        52 => BlsFr::from_be_bytes_mod_order(&[
            73, 215, 210, 192, 180, 73, 229, 23, 155, 197, 204, 195, 180, 76, 96, 117, 217, 132,
            155, 86, 16, 70, 95, 9, 234, 114, 93, 220, 151, 114, 58, 148,
        ]),
        53 => BlsFr::from_be_bytes_mod_order(&[
            100, 194, 15, 185, 13, 122, 0, 56, 49, 117, 124, 196, 198, 34, 111, 110, 73, 133, 252,
            158, 203, 65, 107, 159, 104, 76, 160, 53, 29, 150, 121, 4,
        ]),
        54 => BlsFr::from_be_bytes_mod_order(&[
            89, 207, 244, 13, 232, 59, 82, 180, 27, 196, 67, 215, 151, 149, 16, 215, 113, 201, 64,
            185, 117, 140, 168, 32, 254, 115, 181, 200, 213, 88, 9, 52,
        ]),
        55 => BlsFr::from_be_bytes_mod_order(&[
            83, 219, 39, 49, 115, 12, 57, 176, 78, 221, 135, 95, 227, 183, 200, 130, 128, 130, 133,
            205, 188, 98, 29, 122, 244, 248, 13, 213, 62, 187, 113, 176,
        ]),
        56 => BlsFr::from_be_bytes_mod_order(&[
            27, 16, 187, 122, 130, 175, 206, 57, 250, 105, 195, 162, 173, 82, 247, 109, 118, 57,
            130, 101, 52, 66, 3, 17, 155, 113, 38, 217, 180, 104, 96, 223,
        ]),
        57 => BlsFr::from_be_bytes_mod_order(&[
            86, 27, 96, 18, 214, 102, 191, 225, 121, 196, 221, 127, 132, 205, 209, 83, 21, 150,
            211, 170, 199, 197, 112, 12, 235, 49, 159, 145, 4, 106, 99, 201,
        ]),
        58 => BlsFr::from_be_bytes_mod_order(&[
            15, 30, 117, 5, 235, 217, 29, 47, 199, 156, 45, 247, 220, 152, 163, 190, 209, 179, 105,
            104, 186, 4, 5, 192, 144, 210, 127, 106, 0, 183, 223, 200,
        ]),
        59 => BlsFr::from_be_bytes_mod_order(&[
            47, 49, 63, 175, 13, 63, 97, 135, 83, 122, 116, 151, 163, 180, 63, 70, 121, 127, 214,
            227, 241, 142, 177, 202, 255, 69, 119, 86, 184, 25, 187, 32,
        ]),
        60 => BlsFr::from_be_bytes_mod_order(&[
            58, 92, 187, 109, 228, 80, 180, 129, 250, 60, 166, 28, 14, 209, 91, 197, 92, 173, 17,
            235, 240, 247, 206, 184, 240, 188, 62, 115, 46, 203, 38, 246,
        ]),
        61 => BlsFr::from_be_bytes_mod_order(&[
            104, 29, 147, 65, 27, 248, 206, 99, 246, 113, 106, 239, 189, 14, 36, 80, 100, 84, 192,
            52, 142, 227, 143, 171, 235, 38, 71, 2, 113, 76, 207, 148,
        ]),
        62 => BlsFr::from_be_bytes_mod_order(&[
            81, 120, 233, 64, 245, 0, 4, 49, 38, 70, 180, 54, 114, 127, 14, 128, 167, 184, 242,
            233, 238, 31, 220, 103, 124, 72, 49, 167, 103, 39, 119, 251,
        ]),
        63 => BlsFr::from_be_bytes_mod_order(&[
            61, 171, 84, 188, 155, 239, 104, 141, 217, 32, 134, 226, 83, 180, 57, 214, 81, 186,
            166, 226, 15, 137, 43, 98, 134, 85, 39, 203, 202, 145, 89, 130,
        ]),
        64 => BlsFr::from_be_bytes_mod_order(&[
            75, 60, 231, 83, 17, 33, 143, 154, 233, 5, 248, 78, 170, 91, 43, 56, 24, 68, 139, 191,
            57, 114, 225, 170, 214, 157, 227, 33, 0, 144, 21, 208,
        ]),
        65 => BlsFr::from_be_bytes_mod_order(&[
            6, 219, 251, 66, 185, 121, 136, 77, 226, 128, 211, 22, 112, 18, 63, 116, 76, 36, 179,
            59, 65, 15, 239, 212, 54, 128, 69, 172, 242, 183, 26, 227,
        ]),
        66 => BlsFr::from_be_bytes_mod_order(&[
            6, 141, 107, 70, 8, 170, 232, 16, 198, 240, 57, 234, 25, 115, 166, 62, 184, 210, 222,
            114, 227, 210, 201, 236, 167, 252, 50, 210, 47, 24, 185, 211,
        ]),
        67 => BlsFr::from_be_bytes_mod_order(&[
            76, 92, 37, 69, 137, 169, 42, 54, 8, 74, 87, 211, 177, 217, 100, 39, 138, 204, 126, 79,
            232, 246, 159, 41, 85, 149, 79, 39, 167, 156, 235, 239,
        ]),
        68 => BlsFr::from_be_bytes_mod_order(&[
            108, 186, 197, 225, 112, 9, 132, 235, 195, 45, 161, 91, 75, 185, 104, 63, 170, 186,
            181, 95, 103, 204, 196, 247, 29, 149, 96, 179, 71, 90, 119, 235,
        ]),
        69 => BlsFr::from_be_bytes_mod_order(&[
            70, 3, 196, 3, 187, 250, 154, 23, 115, 138, 92, 98, 120, 234, 171, 28, 55, 236, 48,
            176, 115, 122, 162, 64, 159, 196, 137, 128, 105, 235, 152, 60,
        ]),
        70 => BlsFr::from_be_bytes_mod_order(&[
            104, 148, 231, 226, 43, 44, 29, 92, 112, 167, 18, 166, 52, 90, 230, 177, 146, 169, 200,
            51, 169, 35, 76, 49, 197, 106, 172, 209, 107, 194, 241, 0,
        ]),
        71 => BlsFr::from_be_bytes_mod_order(&[
            91, 226, 203, 188, 68, 5, 58, 208, 138, 250, 77, 30, 171, 199, 243, 210, 49, 238, 167,
            153, 185, 63, 34, 110, 144, 91, 125, 77, 101, 197, 142, 187,
        ]),
        72 => BlsFr::from_be_bytes_mod_order(&[
            88, 229, 95, 40, 123, 69, 58, 152, 8, 98, 74, 140, 42, 53, 61, 82, 141, 160, 247, 231,
            19, 165, 198, 208, 215, 113, 30, 71, 6, 63, 166, 17,
        ]),
        73 => BlsFr::from_be_bytes_mod_order(&[
            54, 110, 191, 175, 163, 173, 56, 28, 14, 226, 88, 201, 184, 253, 252, 205, 184, 104,
            167, 215, 225, 241, 246, 154, 43, 93, 252, 197, 87, 37, 85, 223,
        ]),
        74 => BlsFr::from_be_bytes_mod_order(&[
            69, 118, 106, 183, 40, 150, 140, 100, 47, 144, 217, 124, 207, 85, 4, 221, 193, 5, 24,
            168, 25, 235, 188, 196, 208, 156, 63, 93, 120, 77, 103, 206,
        ]),
        75 => BlsFr::from_be_bytes_mod_order(&[
            57, 103, 143, 101, 81, 47, 30, 228, 4, 219, 48, 36, 244, 29, 63, 86, 126, 246, 109,
            137, 208, 68, 208, 34, 230, 188, 34, 158, 149, 188, 118, 177,
        ]),
        76 => BlsFr::from_be_bytes_mod_order(&[
            70, 58, 237, 29, 47, 31, 149, 94, 48, 120, 190, 91, 247, 191, 196, 111, 192, 235, 140,
            81, 85, 25, 6, 168, 134, 143, 24, 255, 174, 48, 207, 79,
        ]),
        77 => BlsFr::from_be_bytes_mod_order(&[
            33, 102, 143, 1, 106, 128, 99, 192, 213, 139, 119, 80, 163, 188, 47, 225, 207, 130,
            194, 95, 153, 220, 1, 164, 229, 52, 200, 143, 229, 61, 133, 254,
        ]),
        78 => BlsFr::from_be_bytes_mod_order(&[
            57, 208, 9, 148, 168, 165, 4, 106, 27, 199, 73, 54, 62, 152, 167, 104, 227, 77, 234,
            86, 67, 159, 225, 149, 75, 239, 66, 155, 197, 51, 22, 8,
        ]),
        79 => BlsFr::from_be_bytes_mod_order(&[
            77, 127, 93, 205, 120, 236, 233, 169, 51, 152, 77, 227, 44, 11, 72, 250, 194, 187, 169,
            31, 38, 25, 150, 184, 233, 209, 2, 23, 115, 189, 7, 204,
        ]),
        _ => BlsFr::ZERO,
    }
}

/// A channel that can be used to draw random elements from a PoseidonBLS hash.
#[derive(Clone, Default)]
pub struct PoseidonBLSChannel {
    digest: BlsFr,
    pub channel_time: ChannelTime,
}

pub fn poseidon_hash_bls(x: BlsFr, y: BlsFr) -> BlsFr {
    let mut state = [x, y, BlsFr::ZERO];
    poseidon_permute_comp_bls(&mut state);
    state[0] + x
}

pub fn poseidon_permute_comp_bls(state: &mut [BlsFr; 3]) {
    let mut idx = 0;
    mix(state);

    // Full rounds
    for _ in 0..4 {
        round_comp(state, idx, true);
        idx += 3;
    }

    // Partial rounds
    for _ in 0..56 {
        round_comp(state, idx, false);
        idx += 1;
    }

    // Full rounds
    for _ in 0..4 {
        round_comp(state, idx, true);
        idx += 3;
    }
}

#[inline]
fn round_comp(state: &mut [BlsFr; 3], idx: usize, full: bool) {
    if full {
        state[0] += poseidon_comp_consts(idx);
        state[1] += poseidon_comp_consts(idx + 1);
        state[2] += poseidon_comp_consts(idx + 2);
        // Optimize multiplication
        state[0] = state[0] * state[0] * state[0] * state[0] * state[0];
        state[1] = state[1] * state[1] * state[1] * state[1] * state[1];
        state[2] = state[2] * state[2] * state[2] * state[2] * state[2];
    } else {
        state[0] += poseidon_comp_consts(idx);
        state[2] = state[2] * state[2] * state[2] * state[2] * state[2];
    }
    mix(state);
}

#[inline(always)]
fn mix(state: &mut [BlsFr; 3]) {
    state[0] = state[0] + state[1] + state[2];
    state[1] = state[0] + state[1];
    state[2] = state[0] + state[2];
}

pub fn poseidon_hash_many_bls(msgs: &[BlsFr]) -> BlsFr {
    let mut state = [BlsFr::ZERO, BlsFr::ZERO, BlsFr::ZERO];
    let mut iter = msgs.chunks_exact(2);

    for msg in iter.by_ref() {
        state[0] += msg[0];
        state[1] += msg[1];
        poseidon_permute_comp_bls(&mut state);
    }
    let r = iter.remainder();
    if r.len() == 1 {
        state[0] += r[0];
    }
    state[r.len()] += BlsFr::ONE;
    poseidon_permute_comp_bls(&mut state);

    state[0]
}

impl PoseidonBLSChannel {
    pub fn digest(&self) -> BlsFr {
        self.digest
    }
    pub fn update_digest(&mut self, new_digest: BlsFr) {
        self.digest = new_digest;
        self.channel_time.inc_challenges();
    }
    fn draw_felt252(&mut self) -> BlsFr {
        let res = poseidon_hash_bls(self.digest, BlsFr::from(self.channel_time.n_sent as u64));
        self.channel_time.inc_sent();
        res
    }

    // TODO(spapini): Understand if we really need uniformity here.
    /// Generates a close-to uniform random vector of BaseField elements.
    fn draw_base_felts(&mut self) -> [BaseField; 8] {
        let shift = NonZero::new(U256::from_u64(1u64 << 31)).unwrap();

        let mut cur = self.draw_felt252();
        let u32s: [u32; 8usize] = std::array::from_fn(|_| {
            let (quotient, reminder) =
                U256::from_be_slice(&cur.into_bigint().to_bytes_be()).div_rem(&shift);
            cur = BlsFr::from_be_bytes_mod_order(&quotient.to_be_bytes());
            u32::from_str_radix(&reminder.to_string(),16).unwrap()
        });

        u32s.into_iter()
            .map(|x| BaseField::reduce(x as u64))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap()
    }
}

impl Channel for PoseidonBLSChannel {
    const BYTES_PER_HASH: usize = BYTES_PER_FELT252;

    fn trailing_zeros(&self) -> u32 {
        let bytes = self.digest.into_bigint().to_bytes_be();
        u128::from_le_bytes(std::array::from_fn(|i| bytes[i])).trailing_zeros()
    }

    // TODO(spapini): Optimize.
    fn mix_felts(&mut self, felts: &[SecureField]) {
        let shift = BlsFr::from(1u64 << 31);
        let mut res = Vec::with_capacity(felts.len() / 2 + 2);
        res.push(self.digest);
        for chunk in felts.chunks(2) {
            res.push(
                chunk
                    .iter()
                    .flat_map(|x| x.to_m31_array())
                    .fold(BlsFr::default(), |cur, y| {
                        cur * shift + BlsFr::from_be_bytes_mod_order(&y.0.to_be_bytes())
                    }),
            );
        }

        // TODO(spapini): do we need length padding?
        self.update_digest(poseidon_hash_many_bls(&res));
    }

    fn mix_nonce(&mut self, nonce: u64) {
        self.update_digest(poseidon_hash_bls(self.digest, nonce.into()));
    }

    fn draw_felt(&mut self) -> SecureField {
        let felts: [BaseField; FELTS_PER_HASH] = self.draw_base_felts();
        SecureField::from_m31_array(felts[..SECURE_EXTENSION_DEGREE].try_into().unwrap())
    }

    fn draw_felts(&mut self, n_felts: usize) -> Vec<SecureField> {
        let mut felts = iter::from_fn(|| Some(self.draw_base_felts())).flatten();
        let secure_felts = iter::from_fn(|| {
            Some(SecureField::from_m31_array([
                felts.next()?,
                felts.next()?,
                felts.next()?,
                felts.next()?,
            ]))
        });
        secure_felts.take(n_felts).collect()
    }

    fn draw_random_bytes(&mut self) -> Vec<u8> {
        let shift = NonZero::new(U256::from_u64(1u64 << 8)).unwrap();
        let mut cur = self.draw_felt252();
        let bytes: [u8; 31] = std::array::from_fn(|_| {
            let (quotient, reminder) =
                U256::from_be_slice(&cur.into_bigint().to_bytes_be()).div_rem(&shift);
            cur = BlsFr::from_be_bytes_mod_order(&quotient.to_be_bytes());
            u8::from_str_radix(&reminder.to_string(),16).unwrap()
        });
        bytes.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::core::channel::poseidon_bls::PoseidonBLSChannel;
    use crate::core::channel::Channel;
    use crate::core::fields::qm31::SecureField;
    use crate::m31;

    #[test]
    fn test_channel_time() {
        let mut channel = PoseidonBLSChannel::default();

        assert_eq!(channel.channel_time.n_challenges, 0);
        assert_eq!(channel.channel_time.n_sent, 0);

        channel.draw_random_bytes();
        assert_eq!(channel.channel_time.n_challenges, 0);
        assert_eq!(channel.channel_time.n_sent, 1);

        channel.draw_felts(9);
        assert_eq!(channel.channel_time.n_challenges, 0);
        assert_eq!(channel.channel_time.n_sent, 6);
    }

    #[test]
    fn test_draw_random_bytes() {
        let mut channel = PoseidonBLSChannel::default();

        let first_random_bytes = channel.draw_random_bytes();

        // Assert that next random bytes are different.
        assert_ne!(first_random_bytes, channel.draw_random_bytes());
    }

    #[test]
    pub fn test_draw_felt() {
        let mut channel = PoseidonBLSChannel::default();

        let first_random_felt = channel.draw_felt();

        // Assert that next random felt is different.
        assert_ne!(first_random_felt, channel.draw_felt());
    }

    #[test]
    pub fn test_draw_felts() {
        let mut channel = PoseidonBLSChannel::default();

        let mut random_felts = channel.draw_felts(5);
        random_felts.extend(channel.draw_felts(4));

        // Assert that all the random felts are unique.
        assert_eq!(
            random_felts.len(),
            random_felts.iter().collect::<BTreeSet<_>>().len()
        );
    }

    #[test]
    pub fn test_mix_felts() {
        let mut channel = PoseidonBLSChannel::default();
        let initial_digest = channel.digest;
        let felts: Vec<SecureField> = (0..2)
            .map(|i| SecureField::from(m31!(i + 1923782)))
            .collect();

        channel.mix_felts(felts.as_slice());

        assert_ne!(initial_digest, channel.digest);
    }
}
