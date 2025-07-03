pub const WAYDROID_DIR: &str = "/var/lib/waydroid";
pub const WAYDROID_CONFIG: &str = "/var/lib/waydroid/waydroid.cfg";
pub const MAGISK_DIR: &str = "system/etc/init/magisk";
pub const BOOTANIM_RC_PATH: &str = "system/etc/init/bootanim.rc";
pub const BOOTANIM_RC_GZ_PATH: &str = "system/etc/init/bootanim.rc.gz";
pub const BOOTANIM_RC: &str = "service bootanim /system/bin/bootanimation
	class core animation
	user graphics
	group graphics audio
	disabled
	oneshot
	ioprio rt 0
	task_profiles MaxPerformance";
pub const BOOTANIM_RC_MAGISK: &str = "service bootanim /system/bin/bootanimation
	class core animation
	user graphics
	group graphics audio
	disabled
	oneshot
	ioprio rt 0
	task_profiles MaxPerformance

on post-fs-data
	start logd
	exec u:r:su:s0 root root -- /system/etc/init/magisk/magisk64 --auto-selinux --setup-sbin /system/etc/init/magisk
	exec u:r:su:s0 root root -- /system/etc/init/magisk/magiskpolicy --live --magisk \"allow * magisk_file lnk_file *\"
	mkdir /sbin/.magisk 700
	mkdir /sbin/.magisk/mirror 700
	mkdir /sbin/.magisk/block 700
	copy /system/etc/init/magisk/config /sbin/.magisk/config
	rm /dev/.magisk_unblock
	start magisk_service_x
	wait /dev/.magisk_unblock 40
	rm /dev/.magisk_unblock

service magisk_service_x /sbin/magisk --auto-selinux --post-fs-data
	user root
	seclabel u:r:su:s0
	oneshot

service magisk_service_y /sbin/magisk --auto-selinux --service
	class late_start
	user root
	seclabel u:r:su:s0
	oneshot

on property:sys.boot_completed=1
	mkdir /data/adb/magisk 755
	exec u:r:su:s0 root root -- /sbin/magisk --auto-selinux --boot-complete

on property:init.svc.zygote=restarting
	exec u:r:su:s0 root root -- /sbin/magisk --auto-selinux --zygote-restart

on property:init.svc.zygote=stopped
	exec u:r:su:s0 root root -- /sbin/magisk --auto-selinux --zygote-restart";
