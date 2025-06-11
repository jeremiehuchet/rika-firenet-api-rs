FROM node:22@sha256:f627d0e594bbc713de73c2c50d18033c89e114cb1645c3b520b201426cfe1975
WORKDIR /mock
COPY package*.json ./
RUN npm ci --omit=dev
COPY src .
EXPOSE 3000
CMD [ "node", "server.js" ]
