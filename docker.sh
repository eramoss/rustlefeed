# docker buildx build -t rustlefeed .   // Only first time
# mkdir -p ./db
docker volume create rustlefeed_db
docker run -d -p 8000:8000  -v rustlefeed_db:/app/db/ rustlefeed:latest