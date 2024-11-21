FROM node:20@sha256:f99a6bd6fb137cfc683e1ff60e7d67aa8d5fcf45a6a64c01c37b4f433a6238b6
WORKDIR /mock
COPY package*.json ./
RUN npm ci --omit=dev
COPY src .
EXPOSE 3000
CMD [ "node", "server.js" ]
