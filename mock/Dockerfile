FROM node:20@sha256:ad58eb609e8447fa2b6c925e15186682fd86bb00a976ccb04b165243ae4ed92e
WORKDIR /mock
COPY package*.json ./
RUN npm ci --omit=dev
COPY src .
EXPOSE 3000
CMD [ "node", "server.js" ]
