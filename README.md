## Docker Build

`docker build -t selfserv-daemon .`

## Docker Run

`docker run -d --init --net=host -v /path/to/persistant/data:/selfserv --name selfserv-daemon --restart=always selfserv-daemon --token /selfserv/token.key --cert /selfserv/cert.pem --key /selfserv/key.pem`

Place the selfserv service token in `/path/to/persistant/data/token.key`.  The SSL cert and key will be written to the same location and renewed automatically.