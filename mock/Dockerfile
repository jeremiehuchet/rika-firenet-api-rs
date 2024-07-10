FROM node:20@sha256:786005cf39792f7046bcd66491056c26d2dbcc669c072d1a1e4ef4fcdddd26eb
WORKDIR /mock
COPY package*.json ./
RUN npm ci --omit=dev
COPY src .
EXPOSE 3000
CMD [ "node", "server.js" ]
