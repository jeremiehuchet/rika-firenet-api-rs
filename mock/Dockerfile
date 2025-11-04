FROM node:22@sha256:e10b58ffb7b65a99b70330841145beec96088270e971921a2e1c922b31f0bcd6
WORKDIR /mock
COPY package*.json ./
RUN npm ci --omit=dev
COPY src .
EXPOSE 3000
CMD [ "node", "server.js" ]
