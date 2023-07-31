if [ -d "./home/" ]
then
    rm -r ./home/
fi

mkdir ./home/
cp -v -R ./static/ ./home/static/
cp -v -R ./apache/ ./home/apache/ 