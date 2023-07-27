** Build Rust client and server using Cargo **

Kernel Module:
1. Creates a network connection and byte stream between the two clients.

Rust Client:
1. Takes camera input and transforms from YUV image format to a bytestream
2. Sends IOCTL system calls in order to queue and dequeue the buffer for the returning bytestream
3. Displays facetracked images using Linux Video API. 


Rust Server:
1. Receives bytestream and converts to mapped RBG format.
2. Runs Tensorflow for face tracking and sends as bytestream. 

