#![feature(array_chunks, array_from_fn)]
use opencv::core::{flip, Vec3b};
use opencv::videoio::*;
use opencv::{
	prelude::*,
	videoio,
	highgui::*,
};
use std::{fs::File, os::unix::prelude::AsRawFd, str};
mod utils;
use utils::*;
use libc::*;
use std::ptr;
//use tflitec::interpreter::{Interpreter, Options};
use std::net::TcpStream;
use std::io::{self, Write, Read};
use nix::{sys::ioctl, ioctl_read, ioctl_readwrite};

const VIDIOC_QUERYCAP_MAGIC: u8 = 'V' as u8;
const VIDIOC_QUERYCAP_TYPE_MODE: u8 = 0;
const VIDIOC_G_FMT_MAGIC: u8 = 'V' as u8;
const VIDIOC_G_FMT_TYPE_MODE: u8 = 4;
const VIDIOC_S_FMT_MAGIC: u8 = 'V' as u8;
const VIDIOC_S_FMT_TYPE_MODE: u8 = 5;
const VIDIOC_REQBUFS_MAGIC: u8 = 'V' as u8;
const VIDIOC_REQBUFS_TYPE_MODE: u8 = 8;
const VIDIOC_QUERYBUF_MAGIC: u8 = 'V' as u8;
const VIDIOC_QUERYBUF_TYPE_MODE: u8 = 9;
const VIDIOC_STREAMON_MAGIC: u8 = 'V' as u8;
const VIDIOC_STREAMON_TYPE_MODE: u8 = 18;
const VIDIOC_STREAMOFF_MAGIC: u8 = 'V' as u8;
const VIDIOC_STREAMOFF_TYPE_MODE: u8 = 19;
const VIDIOC_QBUF_MAGIC: u8 = 'V' as u8;
const VIDIOC_QBUF_TYPE_MODE: u8 = 15;
const VIDIOC_DQBUF_MAGIC: u8 = 'V' as u8;
const VIDIOC_DQBUF_TYPE_MODE: u8 = 17;

#[repr(C)]
#[derive(Default)]
pub struct v4l2_capability {
    pub driver: [u8; 16],
    pub card: [u8; 32],
    pub bus_info: [u8; 32],
    pub version: u32,
    pub capabilities: u32,
    pub device_caps: u32,
    pub reserved: [u32; 3],
}
#[repr(C)]
struct v4l2_format {
    fmttype: u32,
    fmt: u32,
    width: u32,
    height: u32,
    pixelformat: u32,
    others: [u8; 208 - 5*4],
}
struct v4l2_requestbuffers{
	count: u32,
	fmttype: u32,
	memory: u32,
	reserved: [u32; 2],
}
struct v4l2_timecode {
	fmttype: u32,
	flags: u32,
	frames: u8,
	seconds: u8,		
	minutes: u8,
	hours: u8,
	userbits: [u8; 4],
}
union m1 {
	offset: u32,
	userptr: u32,
	//*planes: v4l2_plane,
	fd: u8,	
}
struct v4l2_buffer {
	index: u32,
	fmttype: u32,
	bytesused: u32,
	flags: u32,
	field: u32,
	timestamp: timeval,
	timecode: v4l2_timecode, 
	sequence: u32,
	memory: u32,
	m: m1,
	length: u32,
	reserved2: u32,
	reserved: u32,
	others: [u32; 100],
}
fn main() {
	//connection 
        let mut stream = TcpStream::connect("127.0.0.1:8888").expect("can't connect");

	//open camera
	let mut cam = videoio::VideoCapture::new(2, videoio::CAP_ANY).unwrap(); // 0 is the default camera
	videoio::VideoCapture::is_opened(&cam).expect("Open camera [FAILED]");
	cam.set(CAP_PROP_FPS, 30.0).expect("Set camera FPS [FAILED]");

	//open device
	let mut file = File::options()
		.write(true)
		.read(true)
		.open("/dev/video2")
		.unwrap();

    	let media_fd = file.as_raw_fd();
    	println!("camera fd = {}", media_fd);
		
	//set capabilities
	ioctl_read!(vidioc_querycap, VIDIOC_QUERYCAP_MAGIC, VIDIOC_QUERYCAP_TYPE_MODE, v4l2_capability);
    	let mut info: v4l2_capability =  Default::default();
    	match unsafe { vidioc_querycap(media_fd, &mut info as *mut v4l2_capability) } {
       		Ok(_) => {
            		println!("get info [OK]");
        	},
       		Err(e) => {
            		println!("get info [FAILED]: {:?}", e);
        	},
    	}
	
	//Check image format
	ioctl_readwrite!(vidioc_g_fmt, VIDIOC_G_FMT_MAGIC, VIDIOC_G_FMT_TYPE_MODE, v4l2_format);
	let mut format = v4l2_format {
		fmttype: 0,
    		fmt: 0,
    		width: 0,
    		height: 0,
    		pixelformat: 0,
    		others: [0; 208 - 5*4],
	};
	format.fmttype = 1; //V4L2_BUF_TYPE_VIDEO_CAPTURE;
	match unsafe { vidioc_g_fmt(media_fd, &mut format as *mut v4l2_format)} {
        	Ok(_) => {
            		println!("get info [OK]");
        	},
        	Err(e) => {
            		println!("get info [FAILED]: {:?}", e);
        	},
    	}

	//Set format
	ioctl_readwrite!(vidioc_s_fmt, VIDIOC_S_FMT_MAGIC, VIDIOC_S_FMT_TYPE_MODE, v4l2_format);
	format.pixelformat = 0x56595559;
	match unsafe { vidioc_s_fmt(media_fd, &mut format as *mut v4l2_format) } {
        	Ok(_) => {
           		println!("get info [OK]");

        	},
        	Err(e) => {
              		println!("get info [FAILED]: {:?}", e);
        	},
    	}
	//inform buffers
	ioctl_readwrite!(vidioc_reqbufs, VIDIOC_REQBUFS_MAGIC, VIDIOC_REQBUFS_TYPE_MODE, v4l2_requestbuffers);
	let mut bufrequest = v4l2_requestbuffers{
		count: 0,
		fmttype: 0,
		memory: 0,
		reserved: [0; 2],
	};	
	bufrequest.fmttype = 1; //V4L2_BUF_TYPE_VIDEO_CAPTURE
	bufrequest.memory = 1; //V4L2_MEMORY_MMAP
	bufrequest.count = 1; 
	match unsafe { vidioc_reqbufs(media_fd, &mut bufrequest as *mut v4l2_requestbuffers) } {
        	Ok(_) => {
           		println!("get info [OK]");

        	},
        	Err(e) => {
              		println!("get info [FAILED]: {:?}", e);
        	},
    	}
	
	//allocate buffers
	ioctl_readwrite!(vidioc_querybuf, VIDIOC_QUERYBUF_MAGIC, VIDIOC_QUERYBUF_TYPE_MODE, v4l2_buffer);
	let mut timec = v4l2_timecode {
		fmttype: 0,
		flags: 0,
		frames: 0,
		seconds: 0,		
		minutes: 0,
		hours: 0,
		userbits: [0; 4],
	};
	let mut m2 = m1 {
		offset: 0,
	};
	let timev = timeval{
		tv_sec: 0,
		tv_usec: 0,
	};
	let mut bufferinfo = v4l2_buffer {
		index: 0,
		fmttype: 0,
		bytesused: 0,
		flags: 0,	
		field: 0,
		timestamp: timev,
		timecode: timec, 
		sequence: 0,
		memory: 0,
		m: m2,
		length: 0,
		reserved2: 0,
		reserved: 0,
		others: [0; 100],
	};
	bufferinfo.fmttype = 1;
	bufferinfo.memory = 1;
	bufferinfo.index = 0;
	match unsafe { vidioc_querybuf(media_fd, &mut bufferinfo as *mut v4l2_buffer) } {
        	Ok(_) => {
           		println!("get info [OK]");

        	},
        	Err(e) => {
              		//println!("get info [FAILED]: {:?}", e);
        	},
    	}
	//mmap
	unsafe {
        	let data = libc::mmap(
          	  	ptr::null_mut(),
          	  	bufferinfo.length.try_into().unwrap(),
          	  	libc::PROT_READ | libc::PROT_WRITE,
          	  	libc::MAP_SHARED,
          		media_fd,
            		bufferinfo.m.offset.into(),
        	);
        	if data == libc::MAP_FAILED {
            		//panic!("cant access data");
        	}
    	}
	//start stream
	ioctl_readwrite!(vidioc_streamon, VIDIOC_STREAMON_MAGIC, VIDIOC_STREAMON_TYPE_MODE, v4l2_requestbuffers);
	match unsafe { vidioc_streamon(media_fd, &mut bufrequest as *mut v4l2_requestbuffers) } {
        	Ok(_) => {
           		println!("get info [OK]");

        	},
        	Err(e) => {
              		//println!("get info [FAILED]: {:?}", e);
        	},
    	}

	
	loop {
		//QBUF
		ioctl_readwrite!(vidioc_qbuf, VIDIOC_QBUF_MAGIC, VIDIOC_QBUF_TYPE_MODE, v4l2_buffer);
		match unsafe { vidioc_qbuf(media_fd, &mut bufferinfo as *mut v4l2_buffer) } {
        		Ok(_) => {
           			println!("get info [OK]");
        		},
        		Err(e) => {
              			//println!("get info [FAILED]: {:?}", e);
        		},
    		}
			
		//TODO buffer->1d translation to send as stream	

		//DQBUF
		ioctl_readwrite!(vidioc_dqbuf, VIDIOC_DQBUF_MAGIC, VIDIOC_DQBUF_TYPE_MODE, v4l2_buffer);
		match unsafe { vidioc_dqbuf(media_fd, &mut bufferinfo as *mut v4l2_buffer) } {
        		Ok(_) => {
           			println!("get info [OK]");
        		},
        		Err(e) => {
              			//println!("get info [FAILED]: {:?}", e);
        		},
    		}

	

		let mut frame = Mat::default();
		cam.read(&mut frame).expect("VideoCapture: read [FAILED]");

		if frame.size().unwrap().width > 0 {
			// flip the image horizontally
			let mut flipped = Mat::default();
			flip(&frame, &mut flipped, 1).expect("flip [FAILED]");
			// resize the image as a square, size is 
			let resized_img = resize_with_padding(&flipped, [192, 192]);

			// turn Mat into Vec<u8>
			let vec_2d: Vec<Vec<Vec3b>> = resized_img.to_vec_2d().unwrap();
			let mut vec_1d: Vec<u8> = vec_2d.iter().flat_map(|v| v.iter().flat_map(|w| w.as_slice())).cloned().collect();
			
			//convert Vec<u8> to bytes
			let bytes: &mut [u8] = &mut vec_1d;
			
			//Write bytes to stream
			let mut buf = [0; 210592];
                	stream.write(bytes).expect("write fail");

			let read = stream.read(&mut buf);
			
			let mut v: Vec<f32> = Vec::new();
			//Convert u8 to f32 https://www.reddit.com/r/rust/comments/tanaxm/mutating_a_buffer_of_u8s_as_f32s_in_place/
			for(idx, chunk) in buf.array_chunks_mut::<4>().enumerate(){
				let mut float = f32::from_ne_bytes(*chunk);
				v.push(float);
			} 

			draw_keypoints(&mut flipped, &v, 0.25);
			//draw_keypoints(&mut flipped, output_tensor.data::<f32>(), 0.25);
			
			imshow("MoveNet", &flipped).expect("imshow [ERROR]");
		}
		// keypress check
		let key = wait_key(1).unwrap();
		if key > 0 && key != 255 {
			//STREAMOFF
			ioctl_readwrite!(vidioc_streamoff, VIDIOC_STREAMOFF_MAGIC, VIDIOC_STREAMOFF_TYPE_MODE, v4l2_requestbuffers);
			match unsafe { vidioc_streamoff(media_fd, &mut bufrequest as *mut v4l2_requestbuffers) } {
        			Ok(_) => {

           				println!("get info [OK]");
        			},
        			Err(e) => {
              				println!("get info [FAILED]: {:?}", e);
        			},
    			}
			break;
		}
	}
}
