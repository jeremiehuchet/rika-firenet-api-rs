FROM node:24@sha256:1de022d8459f896fff2e7b865823699dc7a8d5567507e8b87b14a7442e07f206
WORKDIR /mock
COPY package*.json ./
RUN npm ci --omit=dev
COPY src .
EXPOSE 3000
CMD [ "node", "server.js" ]
