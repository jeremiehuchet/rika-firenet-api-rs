FROM node:24@sha256:00e9195ebd49985a6da8921f419978d85dfe354589755192dc090425ce4da2f7
WORKDIR /mock
COPY package*.json ./
RUN npm ci --omit=dev
COPY src .
EXPOSE 3000
CMD [ "node", "server.js" ]
