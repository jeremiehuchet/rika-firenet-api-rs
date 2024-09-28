FROM node:20@sha256:db5dd2f30cb82a8bdbd16acd4a8f3f2789f5b24f6ce43f98aa041be848c82e45
WORKDIR /mock
COPY package*.json ./
RUN npm ci --omit=dev
COPY src .
EXPOSE 3000
CMD [ "node", "server.js" ]
