FROM node:20@sha256:3496cba96d3548fc484a7febc77edebffcae693aff6a067bc34ec6d8c1f7affe
WORKDIR /mock
COPY package*.json ./
RUN npm ci --omit=dev
COPY src .
EXPOSE 3000
CMD [ "node", "server.js" ]
