declare -a arr=("lame" "portaudio")

BINARY=extraResources/weresocool_server
for i in "${arr[@]}"
do
  if otool -L $BINARY | grep -q $i
  then 
    echo "${i}: Dynamically Linked";
    exit 1
  else
    echo "${i}: Statically Linked";
  fi
done
