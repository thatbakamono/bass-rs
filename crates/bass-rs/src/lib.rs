pub mod sys;

#[cfg(feature = "alac")]
pub mod alac;
#[cfg(feature = "ape")]
pub mod ape;
#[cfg(feature = "cd")]
pub mod cd;
#[cfg(feature = "dsd")]
pub mod dsd;
#[cfg(feature = "enc")]
pub mod enc;
#[cfg(feature = "enc-flac")]
pub mod enc_flac;
#[cfg(feature = "enc-mp3")]
pub mod enc_mp3;
#[cfg(feature = "enc-ogg")]
pub mod enc_ogg;
#[cfg(feature = "enc-opus")]
pub mod enc_opus;
#[cfg(feature = "flac")]
pub mod flac;
#[cfg(feature = "hls")]
pub mod hls;
#[cfg(feature = "midi")]
pub mod midi;
#[cfg(feature = "mix")]
pub mod mix;
#[cfg(feature = "opus")]
pub mod opus;
#[cfg(feature = "wasapi")]
pub mod wasapi;
#[cfg(feature = "webm")]
pub mod webm;
#[cfg(feature = "wma")]
pub mod wma;
#[cfg(feature = "wv")]
pub mod wv;

use std::ffi::c_void;

use bass_sys::{
    Bass, BASS_ATTRIB_BITRATE, BASS_ATTRIB_BUFFER, BASS_ATTRIB_FREQ, BASS_ATTRIB_GRANULE,
    BASS_ATTRIB_NET_RESUME, BASS_ATTRIB_NOBUFFER, BASS_ATTRIB_NORAMP, BASS_ATTRIB_PAN,
    BASS_ATTRIB_SRC, BASS_ATTRIB_VOL, BASS_ERROR_CODEC, BASS_ERROR_FILEFORM, BASS_ERROR_FILEOPEN,
    BASS_ERROR_FORMAT, BASS_ERROR_MEM, BASS_ERROR_NONET, BASS_ERROR_NOPLAY, BASS_ERROR_NOTAUDIO,
    BASS_ERROR_PROTOCOL, BASS_ERROR_SSL, BASS_ERROR_START, BASS_ERROR_TIMEOUT,
    BASS_ERROR_UNSTREAMABLE, BASS_UNICODE, HSTREAM,
};
use once_cell::sync::{Lazy, OnceCell};
use thiserror::Error;

#[cfg(target_family = "windows")]
use widestring::U16CString;

static BASS_PATH: OnceCell<String> = OnceCell::new();
static BASS: Lazy<Bass> = Lazy::new(|| unsafe {
    Bass::new(BASS_PATH.get().expect("BASS_PATH is uninitialized")).expect("Failed to load bass")
});

#[derive(Error, Debug)]
pub enum BassError {
    #[error("The output is paused or stopped.")]
    OutputIsPausedOrStopped,
    #[error("The stream is not playable.")]
    StreamIsNotPlayable,
    #[error("The stream is not playing.")]
    StreamIsNotPlaying,
    #[error("The file couldn't be opened.")]
    FileCouldNotBeOpened,
    #[error("The file format isn't supported or recognised.")]
    InvalidFileFormat,
    #[error("The file doesn't contain audio or it contains audio and video.")]
    InvalidFileContent,
    #[error("The codec isn't supported.")]
    InvalidCodec,
    #[error("The sample format isn't supported.")]
    InvalidSampleFormat,
    #[error("There is too little free memory.")]
    InsufficientMemory,
    #[error("Couldn't initialize 3d support.")]
    CouldNotInitialize3DSupport,
    #[error("Internet connection isn't available.")]
    NoInternetConnection,
    #[error("The protocol isn't supported.")]
    InvalidProtocol,
    #[error("SSL support is not available.")]
    SslSupportNotAvailable,
    #[error("The file can't be streamed.")]
    UnstreamableFile,
    #[error("The server didn't respond to the request within the timeout period.")]
    TimeOut,
}

pub struct Stream {
    handle: HSTREAM,
}

impl Stream {
    pub fn create_from_file(file_name: String) -> Result<Stream, BassError> {
        let handle;

        #[cfg(target_family = "windows")]
        {
            let file_name_raw = U16CString::from_str(file_name).unwrap();
            let file_name_raw = file_name_raw.into_raw() as *const c_void;

            handle = unsafe { BASS.BASS_StreamCreateFile(0, file_name_raw, 0, 0, BASS_UNICODE) };
        }

        #[cfg(target_family = "unix")]
        {
            let file_name_raw = CString::new(file_name).unwrap();
            let file_name_raw = file_name_raw.as_ptr() as *const c_void;

            handle = BASS_StreamCreateFile(0, file_name_raw, 0, 0, 0);
        }

        if handle == 0 {
            let error_code = unsafe { BASS.BASS_ErrorGetCode() } as u32;

            match error_code {
                BASS_ERROR_FILEOPEN => return Err(BassError::FileCouldNotBeOpened),
                BASS_ERROR_FILEFORM => return Err(BassError::InvalidFileFormat),
                BASS_ERROR_NOTAUDIO => return Err(BassError::InvalidFileContent),
                BASS_ERROR_CODEC => return Err(BassError::InvalidCodec),
                BASS_ERROR_FORMAT => return Err(BassError::InvalidSampleFormat),
                BASS_ERROR_MEM => return Err(BassError::InsufficientMemory),
                _ => panic!("Failed to create the stream, error code: {}", unsafe {
                    BASS.BASS_ErrorGetCode()
                }),
            }
        }

        Ok(Stream { handle })
    }

    pub fn create_from_url(url: String) -> Result<Stream, BassError> {
        let handle;

        #[cfg(target_family = "windows")]
        {
            let url_raw = U16CString::from_str(url).unwrap();
            let url_raw = url_raw.into_raw() as *const c_void;

            handle = unsafe { BASS.BASS_StreamCreateFile(0, url_raw, 0, 0, BASS_UNICODE) };

            unsafe { U16CString::from_raw(url_raw as *mut u16) };
        }

        #[cfg(target_family = "unix")]
        {
            let url_raw = CString::new(url).unwrap();
            let url_raw = url_raw.as_ptr() as *const c_void;

            handle = BASS_StreamCreateFile(0, url_raw, 0, 0, 0);

            unsafe { CString::from_raw(url_raw as *mut i8) };
        }

        if handle == 0 {
            let error_code = unsafe { BASS.BASS_ErrorGetCode() } as u32;

            match error_code {
                BASS_ERROR_NONET => return Err(BassError::NoInternetConnection),
                BASS_ERROR_PROTOCOL => return Err(BassError::InvalidProtocol),
                BASS_ERROR_SSL => return Err(BassError::SslSupportNotAvailable),
                BASS_ERROR_TIMEOUT => return Err(BassError::TimeOut),
                BASS_ERROR_FILEOPEN => return Err(BassError::FileCouldNotBeOpened),
                BASS_ERROR_FILEFORM => return Err(BassError::InvalidFileFormat),
                BASS_ERROR_UNSTREAMABLE => return Err(BassError::UnstreamableFile),
                BASS_ERROR_NOTAUDIO => return Err(BassError::InvalidFileContent),
                BASS_ERROR_CODEC => return Err(BassError::InvalidCodec),
                BASS_ERROR_FORMAT => return Err(BassError::InvalidSampleFormat),
                BASS_ERROR_MEM => return Err(BassError::InsufficientMemory),
                _ => panic!("Failed to create the stream, error code: {}", unsafe {
                    BASS.BASS_ErrorGetCode()
                }),
            }
        }

        Ok(Stream { handle })
    }

    pub fn play(&self) -> Result<(), BassError> {
        if unsafe { BASS.BASS_ChannelPlay(self.handle, 0) } == 0 {
            let error_code = unsafe { BASS.BASS_ErrorGetCode() } as u32;

            match error_code {
                BASS_ERROR_START => return Err(BassError::OutputIsPausedOrStopped),
                _ => panic!("Failed to play the stream, error code: {}", error_code),
            }
        }

        Ok(())
    }

    pub fn pause(&self) -> Result<(), BassError> {
        if unsafe { BASS.BASS_ChannelPause(self.handle) } == 0 {
            let error_code = unsafe { BASS.BASS_ErrorGetCode() } as u32;

            match error_code {
                BASS_ERROR_NOPLAY => return Err(BassError::StreamIsNotPlaying),
                _ => panic!("Failed to pause the stream, error code: {}", error_code),
            }
        }

        Ok(())
    }

    pub fn stop(&self) -> Result<(), BassError> {
        if unsafe { BASS.BASS_ChannelStop(self.handle) } == 0 {
            panic!("Failed to stop the stream, error code: {}", unsafe {
                BASS.BASS_ErrorGetCode()
            });
        }

        Ok(())
    }

    pub fn lock(&self) {
        if unsafe { BASS.BASS_ChannelLock(self.handle, 1) } == 0 {
            panic!("Failed to lock the stream, error code: {}", unsafe {
                BASS.BASS_ErrorGetCode()
            });
        }
    }

    pub fn unlock(&self) {
        if unsafe { BASS.BASS_ChannelLock(self.handle, 0) } == 0 {
            panic!("Failed to unlock the stream, error code: {}", unsafe {
                BASS.BASS_ErrorGetCode()
            });
        }
    }

    pub fn get_bit_rate(&self) -> f32 {
        let mut bit_rate = 0.0f32;

        unsafe {
            BASS.BASS_ChannelGetAttribute(
                self.handle,
                BASS_ATTRIB_BITRATE,
                &mut bit_rate as *mut f32,
            )
        };

        bit_rate
    }

    pub fn get_buffering_length(&self) -> f32 {
        let mut buffering_length = 0.0f32;

        unsafe {
            BASS.BASS_ChannelGetAttribute(
                self.handle,
                BASS_ATTRIB_BUFFER,
                &mut buffering_length as *mut f32,
            )
        };

        buffering_length
    }

    pub fn get_sample_rate(&self) -> f32 {
        let mut sample_rate = 0.0f32;

        unsafe {
            BASS.BASS_ChannelGetAttribute(
                self.handle,
                BASS_ATTRIB_FREQ,
                &mut sample_rate as *mut f32,
            )
        };

        sample_rate
    }

    pub fn get_processing_granularity(&self) -> f32 {
        let mut processing_granularity = 0.0f32;

        unsafe {
            BASS.BASS_ChannelGetAttribute(
                self.handle,
                BASS_ATTRIB_GRANULE,
                &mut processing_granularity as *mut f32,
            )
        };

        processing_granularity
    }

    pub fn get_buffer_level_required_to_resume_stalled_playback(&self) -> f32 {
        let mut buffer_level_required_to_resume_stalled_playback = 0.0f32;

        unsafe {
            BASS.BASS_ChannelGetAttribute(
                self.handle,
                BASS_ATTRIB_NET_RESUME,
                &mut buffer_level_required_to_resume_stalled_playback as *mut f32,
            )
        };

        buffer_level_required_to_resume_stalled_playback
    }

    pub fn get_playback_buffering_switch(&self) -> f32 {
        let mut playback_buffering_switch = 0.0f32;

        unsafe {
            BASS.BASS_ChannelGetAttribute(
                self.handle,
                BASS_ATTRIB_NOBUFFER,
                &mut playback_buffering_switch as *mut f32,
            )
        };

        playback_buffering_switch
    }

    pub fn get_playback_ramping_switch(&self) -> f32 {
        let mut playback_ramping_switch = 0.0f32;

        unsafe {
            BASS.BASS_ChannelGetAttribute(
                self.handle,
                BASS_ATTRIB_NORAMP,
                &mut playback_ramping_switch as *mut f32,
            )
        };

        playback_ramping_switch
    }

    pub fn get_panning_position(&self) -> f32 {
        let mut panning_position = 0.0f32;

        unsafe {
            BASS.BASS_ChannelGetAttribute(
                self.handle,
                BASS_ATTRIB_PAN,
                &mut panning_position as *mut f32,
            )
        };

        panning_position
    }

    pub fn get_sample_rate_conversion_quality(&self) -> f32 {
        let mut sample_rate_conversion_quality = 0.0f32;

        unsafe {
            BASS.BASS_ChannelGetAttribute(
                self.handle,
                BASS_ATTRIB_SRC,
                &mut sample_rate_conversion_quality as *mut f32,
            )
        };

        sample_rate_conversion_quality
    }

    pub fn get_volume(&self) -> f32 {
        let mut volume = 0.0f32;

        unsafe {
            BASS.BASS_ChannelGetAttribute(self.handle, BASS_ATTRIB_VOL, &mut volume as *mut f32)
        };

        volume
    }

    pub fn set_buffering_length(&self, value: f32) {
        unsafe { BASS.BASS_ChannelSetAttribute(self.handle, BASS_ATTRIB_BUFFER, value) };
    }

    pub fn set_sample_rate(&self, value: f32) {
        unsafe { BASS.BASS_ChannelSetAttribute(self.handle, BASS_ATTRIB_FREQ, value) };
    }

    pub fn set_processing_granularity(&self, value: f32) {
        unsafe { BASS.BASS_ChannelSetAttribute(self.handle, BASS_ATTRIB_GRANULE, value) };
    }

    pub fn set_buffer_level_required_to_resume_stalled_playback(&self, value: f32) {
        unsafe { BASS.BASS_ChannelSetAttribute(self.handle, BASS_ATTRIB_NET_RESUME, value) };
    }

    pub fn set_playback_buffering_switch(&self, value: f32) {
        unsafe { BASS.BASS_ChannelSetAttribute(self.handle, BASS_ATTRIB_NOBUFFER, value) };
    }

    pub fn set_playback_ramping_switch(&self, value: f32) {
        unsafe { BASS.BASS_ChannelSetAttribute(self.handle, BASS_ATTRIB_NORAMP, value) };
    }

    pub fn set_panning_position(&self, value: f32) {
        unsafe { BASS.BASS_ChannelSetAttribute(self.handle, BASS_ATTRIB_PAN, value) };
    }

    pub fn set_volume(&self, value: f32) {
        unsafe { BASS.BASS_ChannelSetAttribute(self.handle, BASS_ATTRIB_VOL, value) };
    }

    pub fn get_position(&self) -> u64 {
        unsafe { BASS.BASS_ChannelGetPosition(self.handle, 0) }
    }

    pub fn get_time(&self) -> f64 {
        unsafe { BASS.BASS_ChannelBytes2Seconds(self.handle, self.get_position()) }
    }

    pub fn get_raw_handle(&self) -> &HSTREAM {
        &self.handle
    }
}

impl Drop for Stream {
    fn drop(&mut self) {
        unsafe { BASS.BASS_StreamFree(self.handle) };
    }
}
