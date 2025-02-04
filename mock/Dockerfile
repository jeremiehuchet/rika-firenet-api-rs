FROM node:20@sha256:feaa57f4157b2d04b5f2a5a59f0439f7375e88c0da7331db7ddbce731d5f227f
WORKDIR /mock
COPY package*.json ./
RUN npm ci --omit=dev
COPY src .
EXPOSE 3000
CMD [ "node", "server.js" ]
