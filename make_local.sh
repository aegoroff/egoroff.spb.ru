if [ -d "./home/" ]
then
    rm -r ./home/
fi

(cd ./ui/; npm run build)

mkdir ./home/
cp -v -R ./static/ ./home/static/
cp -v -R ./apache/ ./home/apache/

(cd ./egoroff/; cargo clean; cargo b --workspace; ./target/debug/egoroff server)