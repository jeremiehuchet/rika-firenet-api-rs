FROM node:20@sha256:d17aaa2a2fd82e09bd6a6da7cc4a79741340d2a3e39d172d1b30f295b1a850ff
WORKDIR /mock
COPY package*.json ./
RUN npm ci --omit=dev
COPY src .
EXPOSE 3000
CMD [ "node", "server.js" ]
