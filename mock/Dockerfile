FROM node:22@sha256:fbe64f0a038c6117d58d6a160c52994ee586cab9e36812802b564ac2d464bb24
WORKDIR /mock
COPY package*.json ./
RUN npm ci --omit=dev
COPY src .
EXPOSE 3000
CMD [ "node", "server.js" ]
