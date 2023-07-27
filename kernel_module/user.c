

#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <unistd.h>
#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <errno.h>
#include <sys/mman.h>
#define INIT_VAL 0

int main(){
	int *address;
	int fd, off;
	double time;
	long page_size;
	fd = open("/dev/mymem", O_RDWR);
	if(fd < 0){
		printf("mymem could not be opened: errno %d\n", errno);
		return 0;
	}
	page_size = sysconf(_SC_PAGE_SIZE);
	address = mmap(NULL, page_size, PROT_WRITE | PROT_READ, MAP_SHARED, fd, 0);
	address[0] = INIT_VAL;
	char buff[100];
	char one[2] = "1";
	//write(fd, 1, 1);
	//read(fd, buff, 100);
	//printf("%s buff", buff);
	int N = 10004000;
	int W = 100;
	for(int i = 0; i < W; i++){
		pid_t pid = fork();
		if(pid == 0){
			exit(0);
		}
		for(int j = 0; j < N; j++){
			address[0]++;
		}
	}
	printf("%d = %d", address[0], INIT_VAL + (W * N));

	off = lseek(fd, 0, SEEK_SET);


	close(fd);
	return 0;
}

