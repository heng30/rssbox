#/bin/bash

version=`git describe --tags --abbrev=0`
machine=`uname -m`
output_dir=rssbox-win-$machine
target_name=rssbox-$version.exe
target_path=./target/release/rssbox.exe
dll_dir=./win/dll

if [ -f $target_path ]; then
	mkdir -p $output_dir
	cp -f $target_path $output_dir/$target_name
	cp -f $dll_dir/* $output_dir
	tar -zcvf $output_dir.tar.gz $output_dir
else
	echo "Can't find $target_path"
fi
