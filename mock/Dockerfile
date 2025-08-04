FROM node:22@sha256:3218f0d1b9e4b63def322e9ae362d581fbeac1ef21b51fc502ef91386667ce92
WORKDIR /mock
COPY package*.json ./
RUN npm ci --omit=dev
COPY src .
EXPOSE 3000
CMD [ "node", "server.js" ]
