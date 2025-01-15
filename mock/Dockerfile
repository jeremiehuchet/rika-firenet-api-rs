FROM node:20@sha256:2670ef92bde6bcae45ea5e55ce10335bd6592ebbb9bdbfa134cc16f74725c098
WORKDIR /mock
COPY package*.json ./
RUN npm ci --omit=dev
COPY src .
EXPOSE 3000
CMD [ "node", "server.js" ]
