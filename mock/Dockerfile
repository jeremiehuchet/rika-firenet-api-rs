FROM node:22@sha256:7725a5c2c83eed1d36258c66efae14b1ceccd021db9ed1d9559d3335ed3d68ed
WORKDIR /mock
COPY package*.json ./
RUN npm ci --omit=dev
COPY src .
EXPOSE 3000
CMD [ "node", "server.js" ]
