FROM node:22@sha256:23c24e85395992be118734a39903e08c8f7d1abc73978c46b6bda90060091a49
WORKDIR /mock
COPY package*.json ./
RUN npm ci --omit=dev
COPY src .
EXPOSE 3000
CMD [ "node", "server.js" ]
