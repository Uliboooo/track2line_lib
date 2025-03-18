use rodio::Decoder;
use std::{fs, io::BufReader, path::Path};
use whisper_rs::{self, WhisperContext, WhisperContextParameters};

/// langがNoneの場合は"ja"になります。
/// 
/// この関数は正しい結果を返さないため、実験的な機能です。this func is experimental.
#[cfg(feature = "experimental")]
pub fn transcription<P: AsRef<Path>>(model_path: P, audio_path: P, lang: Option<&str>) -> Vec<String> {
    let ctx = WhisperContext::new_with_params(
        model_path.as_ref().to_str().unwrap(),
        WhisperContextParameters::default(),
    )
    .unwrap();

    let mut params =
        whisper_rs::FullParams::new(whisper_rs::SamplingStrategy::Greedy { best_of: 1 });
    params.set_language(Some(lang.unwrap_or("ja")));
    // hide info
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);

    let audio_file = fs::File::open(audio_path).unwrap();
    let audio_data = Decoder::new(BufReader::new(audio_file)).unwrap();
    // let rate = audio_data.sample_rate();
    // println!("original rate: {}", rate);

    let data = audio_data.into_iter().collect::<Vec<i16>>();
    let mut output = vec![0.0f32; data.len()];

    whisper_rs::convert_integer_to_float_audio(&data, &mut output).unwrap();
    // なんかファイルによってエラーになる`value: HalfSampleMissing(113767)`
    let mono_data = whisper_rs::convert_stereo_to_mono_audio(&output).unwrap();

    let mut state = ctx.create_state().unwrap();
    state.full(params, &mono_data).unwrap();

    let num_segments = state.full_n_segments().unwrap();
    
    let mut sc = Vec::new();

    for i in 0..num_segments {
        // let segment = state.full_get_segment_text(i).unwrap();
        // let start_time = state.full_get_segment_t0(i).unwrap();
        // let end_time = state.full_get_segment_t1(i).unwrap();
        // println!("[{} - {}]: {}", start_time, end_time, segment);
        sc.push(state.full_get_segment_text(i).unwrap());
    }
    
    sc
    // println!("{:?}", state);
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::path::PathBuf;

//     #[test]
//     fn test_whisper() {
//         let model_path = PathBuf::from("src/models/ggml-base.bin");

//         let audio_path = PathBuf::from("assets_for_test/source/english_test.mp3");
//         // let audio_path = PathBuf::from("assets_for_test/assets/Talk1_2.wav");
//         // let audio_path = PathBuf::from("assets_for_test/source/001_つくよみちゃん（れいせい）_これはテストです。.wav");
//         // let audio_path = PathBuf::from("assets_for_test/source/but_this_code_dont_compile.wav");
//         // let audio_path = PathBuf::from("assets_for_test/source/but_this_code_dont_compile_48.mp3");
//         if !audio_path.exists() {
//             println!("a");
//             panic!()
//         }
//         let a = transcription(model_path, audio_path, Some("en"));
//         println!("{:?}", a);
//     }
// }
