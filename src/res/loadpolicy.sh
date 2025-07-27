#!/system/bin/sh
MAGISKTMP=/debug_ramdisk
export MAGISKTMP
mount -t devpts -o mode=600,ptmxmode=000,rw,relatime,nosuid,noexec devpts /debug_ramdisk/.magisk/pts
mkdir -p /data/adb/magisk
cp $MAGISKTMP/* /data/adb/magisk/
cp /system/etc/init/magisk/module_installer.sh /data/adb/magisk/
sync
chmod -R 755 /data/adb/magisk
restorecon -R /data/adb/magisk
MAKEDEV=1 $MAGISKTMP/magisk --preinit-device 2>&1
RULESCMD=""
for r in $MAGISKTMP/.magisk/preinit/*/sepolicy.rule; do
    [ -f "$r" ] || continue
    RULESCMD="$RULESCMD --apply $r"
done
$MAGISKTMP/magiskpolicy --live $RULESCMD 2>&1