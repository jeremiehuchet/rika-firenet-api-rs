FROM node:20@sha256:48ca2016597303600e8ff125a42aa3fc7a08325da6615c89dc4cddf156bded6f
WORKDIR /mock
COPY package*.json ./
RUN npm ci --omit=dev
COPY src .
EXPOSE 3000
CMD [ "node", "server.js" ]
