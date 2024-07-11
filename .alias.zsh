resample() {
  if [ $# -lt 1 ]
  then
    echo "Usage: resample <video>"
    return
  fi

#  echo "${1%.*}"
  ffmpeg -i $1 -map 0:1 -b:a 128k -acodec libmp3lame "${1%.*}-resampled.mp3"
}

