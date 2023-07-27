#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/init.h>
#include <linux/fs.h>
#include <linux/kdev_t.h>
#include <linux/device.h>
#include <linux/slab.h>
#include <linux/cdev.h>
#include <linux/mm.h>
#include <asm/io.h>
#define DRIVER_AUTHOR "Daniel Kim <Daniel.Kim.dhk42@yale.edu>"
#define DRIVER_DESC "part 3"
#define DEVICE_NAME "mymem"
#define INIT_VAL 0
static int dev_open(struct inode *, struct file *);
static int dev_release(struct inode *, struct file *);
static ssize_t dev_read(struct file *, char *, size_t, loff_t *);
static ssize_t dev_write(struct file *, const char *, size_t, loff_t *);
static loff_t dev_llseek(struct file *, loff_t, int);
static int dev_mmap(struct file *, struct vm_area_struct *);
//static int dev_fault(struct vm_area_struct *, struct vm_fault *);
dev_t dev = 0; /* dev number */
static struct class *dev_class;
void *dev_buffer; /*buffer we initialize to 512kb*/
void *kmalloc_area;
void *int_buffer;
int deviceOpened = 0; //keep track of devices opened
struct device *device1;
static struct cdev *dev_obj;
/*static struct vm_operations_struct dev_vm_ops = {
	.fault = dev_fault;
};*/
static struct file_operations dev_fops = {
	.owner 	= THIS_MODULE,
	.open	= dev_open,
	.read	= dev_read,
	.write	= dev_write,
	.llseek	= dev_llseek,
	.release= dev_release,
	.mmap	= dev_mmap
};

/* This function is called when module is loaded*/
static int __init mod_init(void){
	int rc = alloc_chrdev_region(&dev, 0, 1, DEVICE_NAME); 
	//major = register_chrdev(0, DEVICE_NAME, &dev_fops);
	dev_buffer = kmalloc(524288, GFP_KERNEL);

	kmalloc_area = (int *)((((unsigned long)(kmalloc(500, GFP_KERNEL))) + PAGE_SIZE -1) & PAGE_MASK);
	//int_array = (int *)(kmalloc(500, GFP_KERNEL));

	//printk("memory %zu  bytes", ksize(dev_buffer));
	if(rc < 0){
		printk(KERN_ALERT "Unable to register device: %d \n", rc);
		return rc;
	}

	dev_obj = cdev_alloc(); //initialize cdev
	cdev_init(dev_obj, &dev_fops); //intialize with fops
	//dev = MKDEV(major, 0);
	rc = cdev_add(dev_obj, dev, 1); //add cdev
	if(rc < 0){
		printk("Unable to add cdev \n");
		unregister_chrdev_region(dev, 1);

	}
	dev_class = class_create(THIS_MODULE, DEVICE_NAME);
	//Create class first before dev
	if(IS_ERR(dev_class)){
		printk(KERN_WARNING "Unable to create class");
		class_destroy(dev_class);
		return -1;
	}
	//Create dev
	device1 = device_create(dev_class, NULL, dev, NULL, "mymem");
	if(IS_ERR(device1)){
		printk(KERN_WARNING "Unable to create device mymem");
		cdev_del(dev_obj); 
		return -EINVAL;
	}
	return 0;
}

/* This function is called when modlle is unloaded */
static void __exit mod_exit(void){
	kfree(dev_buffer);
	//kfree(int_buffer);
	kfree(kmalloc_area);
	device_destroy(dev_class, dev);
	class_destroy(dev_class);
	cdev_del(dev_obj);
	unregister_chrdev(dev, DEVICE_NAME);
}

static int dev_open(struct inode *inodefp, struct file *fp){
	//printk("Open working");
	if(deviceOpened == 1){
		printk("file already opened, please close\n");
		return -1;
	}
	deviceOpened++;
	return 0;
}

static ssize_t dev_read(struct file *fp, char *buff, size_t len, loff_t *off){
	int remaining = 524288 - *off;
	int readable_bytes;
	//printk("You have read");

	if(remaining > len){
		readable_bytes = len;
	}
	else if(remaining < 0){
		return -EINVAL; //This is error number EINVAL
	}
	else{
		readable_bytes = remaining;
	}
	//printk("buffer has value: %s and read bytes; %d", (char *)int_array, 500);
	if(copy_to_user(buff, dev_buffer, readable_bytes)!=0){
		printk("Could not read full file");
		return -1;
	}
	/*if(copy_to_user(buff, kmalloc_area, 8)!=0){
		printk("Could not read full file");
		return -1;
	}*/
	*off += readable_bytes;
	return readable_bytes;
}
static ssize_t dev_write(struct file *fp, const char *buff, size_t len, loff_t *off){
	int remaining = 524288 - *off;
	int writable_bytes;
	//long *temp, *one;

	//printk("You have buffer: %s \n", buff);
	if(remaining > len){
		writable_bytes = len;
	}
	else if(remaining < 0){
		return -EINVAL; //This is error number EINVAL
	}
	else{
		writable_bytes = remaining;
	}
	//printk("int buffer %d:", *(int *)int_buffer);
	
	/*kstrtol((char *)dev_buffer, 10, temp);
	kstrtol(buff, 10,one);
	*temp += *one;
	sprintf(dev_buffer, "%ld", *temp);
	*/
	if(copy_from_user(dev_buffer, buff, writable_bytes)!= 0){
		printk("Could not write full amount, overflow");
		return -1;
	}
	//printk("buffer: %s\n", (char*)dev_buffer);
	*off += writable_bytes;
	return writable_bytes;
}
static loff_t dev_llseek(struct file *fp, loff_t off, int flag){
	loff_t new_off;
	if(flag == 0){ //seek set
		new_off = off;
	}
	else if(flag == 1){ //seek cur
		new_off = fp->f_pos + off;
	}
	else if(flag == 2){ //seek end
		new_off = 524288 - off;
	}
	fp->f_pos = new_off;
	return off;
}
static int dev_release(struct inode *, struct file *){
	//printk("CLOSE WORKING");
	if(deviceOpened != 1){
		printk("file cannot be closed, file was not opened");
		return -1;
	}
	deviceOpened--;
	return 0;
}
static int dev_mmap(struct file *fp, struct vm_area_struct *vma){
	//vma->vm_ops = &dev_vm_ops;
	long offset = vma->vm_end - vma->vm_start;
	remap_pfn_range(vma, vma->vm_start, virt_to_phys((void *)kmalloc_area) >> PAGE_SHIFT, offset, vma->vm_page_prot);
	/*unsigned long offset = vma->vm_pgoff << PAGE_SHIFT;
	if(remap_pfn_range(vma->vm_start, offset, vma->vm_end-vma->vm_start, vma->vm_page_prot))
		return -EAGAIN;
	*/
	return 0;
}
/*
static int dev_fault(struct vm_area_ststruct vm_fault *vm_f){
	vm_f->page = my_page_at_index(vm_f->pgoff);
	get_page(vm_f->page);
	return 0;
}*/

module_init(mod_init);
module_exit(mod_exit);
MODULE_LICENSE("GPL");
MODULE_AUTHOR(DRIVER_AUTHOR);
MODULE_DESCRIPTION(DRIVER_DESC);

