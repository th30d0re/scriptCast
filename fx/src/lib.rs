//! scriptcast-fx: loudness metering and fixed effect chains over one audio file.
//!
//! The DSP is soundcraft's. This crate only picks plugins, sets parameters, runs them
//! offline with `apply_plugin` and writes a 32-bit float WAV. It does not stitch or mix.

use std::path::Path;

use soundcraft_audio_io::{AudioBuffer, BitDepth, EncodeOptions, FileFormat};
use soundcraft_dsp::Plugin;
use soundcraft_dsp::meter::LoudnessMeter;

/// A named effect chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Preset {
    Speaker,
    Master,
}

impl Preset {
    pub fn parse(s: &str) -> Result<Self, String> {
        match s {
            "speaker" => Ok(Self::Speaker),
            "master" => Ok(Self::Master),
            other => Err(format!("unknown preset '{other}' (use speaker or master)")),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Speaker => "speaker",
            Self::Master => "master",
        }
    }
}

/// The plugin ids of a chain, in processing order.
pub fn chain_ids(preset: Preset, room: bool, eq7: bool) -> Result<Vec<&'static str>, String> {
    match preset {
        Preset::Speaker => {
            let mut ids = vec!["channel_strip", "de_esser"];
            if room {
                ids.push("room_reverb");
            }
            if eq7 {
                ids.push("eq_7band");
            }
            Ok(ids)
        }
        Preset::Master => {
            if room {
                return Err("--room is not allowed with --preset master".into());
            }
            if eq7 {
                return Err("--eq7 is not allowed with --preset master".into());
            }
            Ok(vec!["maximizer", "dither", "dc_offset_removal"])
        }
    }
}

/// One `--set plugin.param=value` override.
#[derive(Debug, Clone, PartialEq)]
pub struct Setting {
    pub plugin: String,
    pub param: String,
    pub value: f32,
}

impl Setting {
    pub fn parse(s: &str) -> Result<Self, String> {
        let bad = || format!("--set '{s}' is not plugin.param=value");
        let (key, value) = s.split_once('=').ok_or_else(bad)?;
        let (plugin, param) = key.split_once('.').ok_or_else(bad)?;
        if plugin.is_empty() || param.is_empty() {
            return Err(bad());
        }
        let value: f32 = value.trim().parse().map_err(|_| format!("--set '{s}': '{value}' is not a number"))?;
        if !value.is_finite() {
            return Err(format!("--set '{s}': value must be finite"));
        }
        Ok(Self { plugin: plugin.to_string(), param: param.to_string(), value })
    }
}

/// Creates the plugins of a chain on their defaults, then applies the overrides.
///
/// An override must name a plugin in this chain and a parameter that plugin has, and its value
/// must be inside the parameter's range.
pub fn build_chain(ids: &[&str], sets: &[Setting]) -> Result<Vec<Box<dyn Plugin>>, String> {
    let mut chain = Vec::with_capacity(ids.len());
    for id in ids {
        let plugin = soundcraft_dsp::create(id).ok_or_else(|| format!("soundcraft has no plugin '{id}'"))?;
        chain.push(plugin);
    }
    for s in sets {
        let info = soundcraft_dsp::plugin_info(&s.plugin).ok_or_else(|| format!("unknown plugin id '{}'", s.plugin))?;
        let pos = ids
            .iter()
            .position(|id| *id == s.plugin)
            .ok_or_else(|| format!("plugin '{}' is not in this chain ({})", s.plugin, ids.join(", ")))?;
        let param = info.param(&s.param).ok_or_else(|| {
            let known: Vec<&str> = info.params.iter().map(|p| p.id).collect();
            format!("unknown param '{}' on {} (params: {})", s.param, s.plugin, known.join(", "))
        })?;
        if s.value < param.min || s.value > param.max {
            return Err(format!("{}.{}={} is outside {}..{}", s.plugin, s.param, s.value, param.min, param.max));
        }
        let plugin = chain.get_mut(pos).ok_or_else(|| format!("plugin '{}' is missing", s.plugin))?;
        if !plugin.set_param(&s.param, s.value) {
            return Err(format!("{} refused param '{}'", s.plugin, s.param));
        }
    }
    Ok(chain)
}

/// Runs each plugin over the whole buffer in order. The frame count does not change.
pub fn run_chain(buf: &mut AudioBuffer, chain: &mut [Box<dyn Plugin>]) {
    let sr = buf.sample_rate as f32;
    for plugin in chain {
        soundcraft_dsp::offline::apply_plugin(&mut buf.channels, plugin.as_mut(), sr);
    }
}

/// BS.1770 loudness of a whole buffer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Loudness {
    pub integrated_lufs: f32,
    pub short_term_lufs: f32,
    pub momentary_lufs: f32,
    pub true_peak_db: f32,
}

pub fn measure(buf: &AudioBuffer) -> Result<Loudness, String> {
    if buf.num_channels() == 0 {
        return Err("the input has no audio channels".into());
    }
    let mut meter = LoudnessMeter::new(buf.sample_rate as f32, buf.num_channels());
    meter.process(&buf.channels, buf.frames());
    Ok(Loudness {
        integrated_lufs: meter.integrated_lufs(),
        short_term_lufs: meter.short_term_lufs(),
        momentary_lufs: meter.momentary_lufs(),
        true_peak_db: meter.true_peak_db(),
    })
}

/// Decodes any file soundcraft can read.
pub fn read_audio(path: &Path) -> Result<AudioBuffer, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let hint = path.extension().and_then(|e| e.to_str());
    let (_info, buf) = soundcraft_audio_io::decode(&bytes, hint).map_err(|e| format!("cannot decode {}: {e}", path.display()))?;
    if buf.sample_rate == 0 {
        return Err(format!("{} has no sample rate", path.display()));
    }
    Ok(buf)
}

/// Writes a 32-bit float WAV at the buffer's own rate and channel count.
pub fn write_wav(path: &Path, buf: &AudioBuffer) -> Result<(), String> {
    let opts = EncodeOptions { format: FileFormat::Wav, bit_depth: BitDepth::Float32, dither: false, bwf: None };
    let bytes = soundcraft_audio_io::encode(buf, &opts).map_err(|e| format!("cannot encode {}: {e}", path.display()))?;
    std::fs::write(path, bytes).map_err(|e| format!("cannot write {}: {e}", path.display()))
}

/// A JSON string literal.
pub fn json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if u32::from(c) < 0x20 => out.push_str(&format!("\\u{:04x}", u32::from(c))),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// A JSON number, or null when the value is not finite.
pub fn json_num(v: f32) -> String {
    if v.is_finite() { format!("{v}") } else { "null".into() }
}

impl Loudness {
    pub fn to_json(&self) -> String {
        format!(
            "{{\"integrated_lufs\":{},\"short_term_lufs\":{},\"momentary_lufs\":{},\"true_peak_db\":{}}}",
            json_num(self.integrated_lufs),
            json_num(self.short_term_lufs),
            json_num(self.momentary_lufs),
            json_num(self.true_peak_db),
        )
    }
}

/// What `chain --json` reports.
#[derive(Debug, Clone, PartialEq)]
pub struct ChainReport {
    pub preset: Preset,
    pub plugins: Vec<&'static str>,
    pub bypass: bool,
    pub frames: usize,
    pub sample_rate: u32,
    pub wrote: String,
}

impl ChainReport {
    pub fn to_json(&self) -> String {
        let plugins: Vec<String> = self.plugins.iter().map(|p| json_str(p)).collect();
        format!(
            "{{\"preset\":{},\"plugins\":[{}],\"bypass\":{},\"frames\":{},\"sample_rate\":{},\"wrote\":{}}}",
            json_str(self.preset.name()),
            plugins.join(","),
            self.bypass,
            self.frames,
            self.sample_rate,
            json_str(&self.wrote),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_chain_plugin_exists() {
        for id in ["channel_strip", "de_esser", "room_reverb", "eq_7band", "maximizer", "dither", "dc_offset_removal"] {
            assert!(soundcraft_dsp::plugin_info(id).is_some(), "{id}");
        }
    }

    #[test]
    fn speaker_order_and_options() {
        assert_eq!(chain_ids(Preset::Speaker, false, false).unwrap(), ["channel_strip", "de_esser"]);
        assert_eq!(chain_ids(Preset::Speaker, true, true).unwrap(), ["channel_strip", "de_esser", "room_reverb", "eq_7band"]);
    }

    #[test]
    fn master_rejects_room_and_eq7() {
        assert_eq!(chain_ids(Preset::Master, false, false).unwrap(), ["maximizer", "dither", "dc_offset_removal"]);
        assert!(chain_ids(Preset::Master, true, false).is_err());
        assert!(chain_ids(Preset::Master, false, true).is_err());
    }

    #[test]
    fn set_parses_and_validates() {
        let s = Setting::parse("channel_strip.hpf_on=1").unwrap();
        assert_eq!((s.plugin.as_str(), s.param.as_str(), s.value), ("channel_strip", "hpf_on", 1.0));
        assert!(Setting::parse("channel_strip=1").is_err());
        assert!(Setting::parse("channel_strip.hpf_on=x").is_err());
        assert!(Setting::parse("channel_strip.hpf_on=nan").is_err());

        let ids = chain_ids(Preset::Speaker, false, false).unwrap();
        assert!(build_chain(&ids, &[s]).is_ok());
        for bad in ["nope.x=1", "channel_strip.nope=1", "maximizer.ceiling=-1", "channel_strip.hpf_freq=99999"] {
            assert!(build_chain(&ids, &[Setting::parse(bad).unwrap()]).is_err(), "{bad}");
        }
    }

    #[test]
    fn json_strings_are_escaped() {
        assert_eq!(json_str("a\"b\\c\n"), "\"a\\\"b\\\\c\\n\"");
        assert_eq!(json_num(f32::NAN), "null");
    }
}
