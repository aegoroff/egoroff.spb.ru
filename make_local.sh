if [ -d "./home/" ]
then
    rm -r ./home/
fi

mkdir ./home/
cp -v -R ./static/ ./home/static/
cp -v -R ./apache/ ./home/apache/ 
find ./home/static/dist/**/*_r.html | sed -r -e 's/((.+)_r.html)/\1 \2.html/g' | xargs -I % bash -c 'mv -v -f %'
find ./home/static/dist/*_r.html | sed -r -e 's/((.+)_r.html)/\1 \2.html/g' | xargs -I % bash -c 'mv -v -f %'